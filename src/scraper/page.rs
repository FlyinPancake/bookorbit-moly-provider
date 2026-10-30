//! The search page and the book page

use color_eyre::eyre::{self, OptionExt};
use scraper::{ElementRef, Html};

use super::{
    MolySlug, ScrapedBook, ScrapedSeries,
    edition::parse_printing,
    language::{language_code, language_tag, split_title_language},
};

static_selector!(SEARCH_RESULT, "a.book_selector");
// "Nincsen találat." when nothing at all matched
static_selector!(SEARCH_NOTICE, "div.search_area > p");
// a results section with no hits is its heading followed by "Nincs találat."
static_selector!(EMPTY_SECTION, "h3 + div.hint");
static_selector!(TITLE, "h1 > span.item");
static_selector!(SERIES, "a.action");
static_selector!(AUTHORS, "div.authors > a");
static_selector!(SUBTITLE, "div.subtitle");
static_selector!(DESCRIPTION, "div.text");
static_selector!(FULL_DESCRIPTION, "#full_description");
static_selector!(DESCRIPTION_PARAGRAPH, "p:not(.spoiler)");
// older pages list editions in `div.items`, newer ones in a timeline
static_selector!(EDITIONS, r#"div.edition[id^="edition_"]"#);
static_selector!(TAGS, "#book_tags > * > a.tag");
static_selector!(RATING, "h1 span.rating .like_count");
static_selector!(RATING_COUNT, "h1 a.statistic_link");

// "Dune 1." has an index, an omnibus such as "A Dűne 1–3." does not
static_regex!(SERIES_INDEX, r"\s(\d+)\.$");
static_regex!(PERCENT, r"(\d+)\s*%");

pub(super) fn parse_search_page(body: &str) -> eyre::Result<Vec<MolySlug>> {
    let document = Html::parse_document(body);
    let slugs = document
        .select(&SEARCH_RESULT)
        .map(|book| {
            Ok(book
                .value()
                .attr("href")
                .ok_or_eyre("book link not found")?
                .replace("/konyvek/", ""))
        })
        .collect::<eyre::Result<Vec<_>>>()?;
    // without a "no hits" notice, an empty list means the page layout changed
    if slugs.is_empty() && !has_no_book_hits(&document) {
        eyre::bail!("search page has neither book links nor a no-hits notice");
    }
    Ok(slugs)
}

fn has_no_book_hits(document: &Html) -> bool {
    let text = |el: ElementRef| clean(&el.text().collect::<String>());
    let nothing_matched = document
        .select(&SEARCH_NOTICE)
        .any(|p| text(p).starts_with("Nincsen találat"));
    let no_books = document.select(&EMPTY_SECTION).any(|hint| {
        hint.prev_siblings()
            .find_map(ElementRef::wrap)
            .is_some_and(|heading| text(heading) == "Könyvek")
    });
    nothing_matched || no_books
}

pub(super) fn clean(s: &str) -> String {
    s.replace('\u{200b}', "").trim().to_string()
}

pub(super) fn parse_book_page(body: &str, slug: MolySlug) -> eyre::Result<ScrapedBook> {
    let document = Html::parse_document(body);
    let title_node = document
        .select(&TITLE)
        .next()
        .ok_or_eyre("title not found")?;

    let full_title = clean(
        title_node
            .first_child()
            .ok_or_eyre("title node not found")?
            .value()
            .as_text()
            .ok_or_eyre("title is not text")?,
    );
    let (title, title_language) = split_title_language(&full_title);
    let title = title.to_string();

    let series = title_node
        .select(&SERIES)
        .map(|series| {
            let series = series
                .text()
                .collect::<String>()
                .trim_matches(['(', ')'].as_ref())
                .to_string();
            match SERIES_INDEX.captures(&series) {
                Some(cap) => ScrapedSeries {
                    name: series[..cap.get(0).unwrap().start()].trim().to_string(),
                    idx: cap.get(1).unwrap().as_str().parse().ok(),
                },
                None => ScrapedSeries {
                    name: series.trim().to_string(),
                    idx: None,
                },
            }
        })
        .collect();

    let authors = document
        .select(&AUTHORS)
        .map(|el| clean(&el.text().collect::<String>()))
        .filter(|a| !a.is_empty())
        .collect();

    let subtitle = document
        .select(&SUBTITLE)
        .next()
        .map(|el| clean(&el.text().collect::<String>()))
        .filter(|s| !s.is_empty());

    let description = document
        .select(&DESCRIPTION)
        .next()
        .map(|el| {
            if let Some(full) = document.select(&FULL_DESCRIPTION).next() {
                full.text().collect::<String>()
            } else {
                // Exclude spoiler warning paragraph if present
                el.select(&DESCRIPTION_PARAGRAPH)
                    .map(|p| p.text().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        })
        .map(|s| clean(&s.replace("\n\n", "\n")))
        .filter(|s| !s.is_empty());

    let printings = document
        .select(&EDITIONS)
        .filter_map(|edition| match parse_printing(edition) {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::warn!(error = %e, slug, "Skipping edition due to parse error");
                None
            }
        })
        .collect();

    let (language_tags, tags): (Vec<String>, Vec<String>) = document
        .select(&TAGS)
        .map(|el| clean(&el.text().collect::<String>()))
        .partition(|tag| language_tag(tag).is_some());

    // A book tagged with several languages (bilingual, or mis-tagged) gets none rather than a guess
    let mut tag_languages: Vec<_> = language_tags
        .iter()
        .filter_map(|tag| language_code(language_tag(tag)?))
        .collect();
    tag_languages.sort_unstable();
    tag_languages.dedup();
    let language = match tag_languages.as_slice() {
        [code] => Some(*code),
        [] => title_language,
        _ => None,
    };

    // moly shows the average rating as a percentage
    let rating = document
        .select(&RATING)
        .next()
        .and_then(|el| {
            PERCENT
                .captures(&el.text().collect::<String>())?
                .get(1)?
                .as_str()
                .parse::<f64>()
                .ok()
        })
        .filter(|p| (0.0..=100.0).contains(p))
        .map(|p| p / 20.0);

    let rating_count = document.select(&RATING_COUNT).next().and_then(|el| {
        el.text()
            .collect::<String>()
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()
    });

    Ok(ScrapedBook {
        authors,
        slug,
        title,
        subtitle,
        series,
        description,
        printings,
        tags,
        language,
        rating,
        rating_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEARCH_PAGE: &str = include_str!("../../tests/fixtures/search_dune.html");
    const EMPTY_SEARCH_PAGE: &str = include_str!("../../tests/fixtures/search_empty.html");
    const BOOK_PAGE: &str = include_str!("../../tests/fixtures/book_dune.html");
    const SLUG: &str = "frank-herbert-dune-angol";

    fn dune() -> ScrapedBook {
        parse_book_page(BOOK_PAGE, SLUG.to_string()).unwrap()
    }

    #[test]
    fn search_page_yields_slugs() {
        let slugs = parse_search_page(SEARCH_PAGE).unwrap();

        assert!(!slugs.is_empty());
        assert!(slugs.contains(&SLUG.to_string()));
        // slugs must be bare, without the /konyvek/ prefix
        assert!(slugs.iter().all(|slug| !slug.contains('/')));
    }

    #[test]
    fn search_without_hits_is_empty() {
        assert!(parse_search_page(EMPTY_SEARCH_PAGE).unwrap().is_empty());
    }

    #[test]
    fn search_with_no_book_hits_is_empty() {
        let page = r#"<table><tr><td><h3>Könyvek </h3><div class="hint">Nincs találat.</div></td>
            <td><h3>Tagok <span class="counter">1</span></h3><div class="user item">x</div></td></tr></table>"#;

        assert!(parse_search_page(page).unwrap().is_empty());
    }

    #[test]
    fn search_page_without_results_or_notice_is_an_error() {
        // other sections being empty says nothing about the books section
        let page = r#"<h3>Alkotók </h3><div class="hint">Nincs találat.</div>"#;

        assert!(parse_search_page(page).is_err());
    }

    #[test]
    fn book_page_title_is_cleaned() {
        let book = dune();

        // zero-width spaces, series link and language suffix stripped
        assert_eq!(book.title, "Dune");
        assert_eq!(book.slug, SLUG);
    }

    #[test]
    fn book_page_parses_authors_and_tags() {
        let book = dune();

        assert_eq!(book.authors, vec!["Frank Herbert".to_string()]);
        assert!(book.tags.contains(&"sci-fi".to_string()));
        assert!(!book.tags.iter().any(|t| t.ends_with("nyelvű")));
        assert_eq!(book.language, Some("en"));
    }

    #[test]
    fn book_page_parses_series_with_index() {
        let book = dune();

        assert_eq!(book.series.len(), 1);
        assert_eq!(book.series[0].name, "Dune");
        assert_eq!(book.series[0].idx, Some(1));
    }

    #[test]
    fn book_page_parses_rating() {
        let book = dune();

        assert_eq!(book.rating, Some(4.5));
        assert_eq!(book.rating_count, Some(138));
    }

    #[test]
    fn book_page_parses_printings() {
        let book = dune();

        assert!(!book.printings.is_empty());
        for printing in &book.printings {
            assert!(!printing.publisher.is_empty());
            assert_eq!(printing.publishing_year.len(), 4);
            assert!(
                printing
                    .cover_url
                    .as_deref()
                    .is_none_or(|url| url.contains("/system/covers/big/"))
            );
            assert!(printing.isbn.as_deref().is_none_or(|i| !i.contains('-')));
        }
        // at least one edition carries an ISBN and one carries an ASIN
        assert!(book.printings.iter().any(|pt| pt.isbn.is_some()));
        assert!(book.printings.iter().any(|pt| pt.asin.is_some()));

        let first = &book.printings[0];
        assert_eq!(first.publisher, "Hodder & Stoughton");
        assert_eq!(first.publishing_year, "2021");
        assert_eq!(first.pages, Some(578));
        assert_eq!(first.isbn.as_deref(), Some("9781529347869"));
        assert!(!first.is_audiobook);
    }

    #[test]
    fn book_page_parses_published_date() {
        let book = dune();

        let ace = book
            .printings
            .iter()
            .find(|p| p.isbn.as_deref() == Some("9780593200117"))
            .unwrap();
        assert_eq!(
            ace.published_date,
            chrono::NaiveDate::from_ymd_opt(2020, 11, 10)
        );
    }

    #[test]
    fn omnibus_series_has_no_index() {
        let page = r#"<h1><span class="item">Frank Herbert teljes science fiction univerzuma 1. <a class="action" href="/sorozatok/a-dune">(A Dűne 1–3.)</a></span></h1>"#;
        let book = parse_book_page(page, "x".to_string()).unwrap();

        assert_eq!(
            book.series,
            vec![ScrapedSeries {
                name: "A Dűne 1–3.".to_string(),
                idx: None
            }]
        );
    }

    #[test]
    fn audiobook_edition_in_flex_layout() {
        let page = r#"<h1><span class="item">Harry Potter</span></h1>
            <div class="items"><div id="edition_141496" class="flex edition edition_141496">
            <div class="flex_cover"><a href="/kiadasok/141496"><img alt="" data-cover="/system/covers/normal/covers_144289.jpg?1586181805" class="screenshot" src="https://assets.moly.hu/system/covers/normal/covers_144289.jpg?1586181805" /></a> </div>
            <div class="flex_content"><a href="/kiadasok/141496">x</a><div><a href="/kiadok/animus">Animus</a>, Budapest, 2006 <img title="Hangoskönyv" class="tooltip" alt="" src="x.png" /> <a class="tag" href="/cimkek/hangoskonyv">hangoskönyv</a> </div>
            <div><strong>Fordította</strong>: <a href="/alkotok/toth-tamas-boldizsar">Tóth Tamás Boldizsár</a><span class="dot"> · </span><strong>Felolvasta</strong>: <a href="/alkotok/kern-andras">Kern András</a></div></div></div></div>"#;
        let book = parse_book_page(page, "hp".to_string()).unwrap();
        let printing = &book.printings[0];

        assert!(printing.is_audiobook);
        assert_eq!(printing.narrators, vec!["Kern András".to_string()]);
        assert_eq!(printing.publisher, "Animus");
        assert_eq!(printing.publishing_year, "2006");
        assert_eq!(
            printing.cover_url.as_deref(),
            Some("https://moly.hu/system/covers/big/covers_144289.jpg?1586181805")
        );
    }

    #[test]
    fn timeline_page_finds_audiobook_edition() {
        let page = include_str!("../../tests/fixtures/book_harry_potter.html");
        let book = parse_book_page(
            page,
            "j-k-rowling-harry-potter-es-a-bolcsek-kove".to_string(),
        )
        .unwrap();

        assert!(book.printings.len() > 50);
        let audiobook = book.printings.iter().find(|p| p.is_audiobook).unwrap();
        assert_eq!(audiobook.narrators, vec!["Kern András".to_string()]);
        assert_eq!(audiobook.publishing_year, "2006");
    }

    #[test]
    fn hungarian_book_language_comes_from_tag() {
        let page = include_str!("../../tests/fixtures/book_harry_potter.html");
        let book = parse_book_page(page, "hp".to_string()).unwrap();

        assert_eq!(book.language, Some("hu"));
    }

    #[test]
    fn title_language_is_a_fallback_for_missing_tags() {
        let page = r#"<h1><span class="item">Dune (angol)</span></h1>"#;
        let book = parse_book_page(page, "x".to_string()).unwrap();

        assert_eq!(book.title, "Dune");
        assert_eq!(book.language, Some("en"));
    }

    #[test]
    fn conflicting_language_tags_give_no_language() {
        let page = r#"<h1><span class="item">Kétnyelvű</span></h1><div id="book_tags"><span><a class="tag">angol nyelvű</a></span><span><a class="tag">magyar nyelvű</a></span></div>"#;
        let book = parse_book_page(page, "x".to_string()).unwrap();

        assert_eq!(book.language, None);
        assert!(book.tags.is_empty());
    }

    #[test]
    fn book_page_parses_description() {
        let description = dune().description.expect("description missing");

        assert!(!description.is_empty());
        assert!(!description.contains('\u{200b}'));
    }

    #[test]
    fn parse_garbage_html_does_not_panic() {
        assert!(parse_book_page("<html></html>", "whatever".to_string()).is_err());
        assert!(parse_search_page("<html></html>").is_err());
    }
}
