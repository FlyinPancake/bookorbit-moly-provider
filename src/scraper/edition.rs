//! One edition ("kiadás") listed on a book page

use std::str::FromStr;

use color_eyre::eyre::{self, OptionExt};
use isbn2::Isbn;
use scraper::ElementRef;

use super::{BASE_URL, ScrapedPrinting};

static_selector!(LINK, "a");
static_selector!(DIV, "div");
static_selector!(ABBR, "abbr");
static_selector!(COVER, "img.screenshot");

static_regex!(YEAR, r"\b(19|20)\d{2}\b");
static_regex!(DATE, r"(\d{4})\.\s+(\p{L}+)\s+(\d{1,2})\.");
static_regex!(ISBN, r"ISBN[:\s]*([0-9Xx\-]{10,17})");
static_regex!(ASIN, r"ASIN[:\s]*([A-Z0-9]{8,12})");
static_regex!(NARRATOR, r"Felolvasta[:\s]*([^·\n]+)");
static_regex!(PAGES, r"(\d+)\s*oldal");

pub(super) fn parse_printing(edition: ElementRef) -> eyre::Result<ScrapedPrinting> {
    let edition_text = edition.text().collect::<String>();

    let publisher = edition
        .select(&LINK)
        .find(|el| {
            el.value()
                .attr("href")
                .is_some_and(|h| h.contains("/kiadok/"))
        })
        .ok_or_else(|| eyre::eyre!("publisher not found in edition: {}", edition_text.trim()))?
        .text()
        .collect::<String>();

    // The line naming the publisher also carries the year. Newer layouts put the cover in a
    // div before it, so it is not always the first one.
    let first_line = edition.select(&DIV).find(|d| {
        d.children().filter_map(ElementRef::wrap).any(|el| {
            el.value()
                .attr("href")
                .is_some_and(|h| h.contains("/kiadok/"))
        })
    });
    let first_line_text = first_line
        .map(|d| d.text().collect::<String>())
        .unwrap_or_else(|| edition_text.clone());
    let last_year = |s: &str| {
        YEAR.captures_iter(s)
            .last()
            .map(|c| c.get(0).unwrap().as_str().to_string())
    };
    let publishing_year = last_year(&first_line_text)
        .or_else(|| last_year(&edition_text))
        .ok_or_eyre("publishing_year not found")?;

    // The exact date sits in a tooltip: "Megjelenés időpontja: 2020. november 10."
    let published_date = edition
        .select(&ABBR)
        .next()
        .and_then(|abbr| abbr.value().attr("title"))
        .and_then(parse_hungarian_date);

    let isbn = ISBN
        .captures(&edition_text)
        .and_then(|cap| Isbn::from_str(cap.get(1)?.as_str().trim()).ok())
        .map(|isbn| isbn.to_string().replace('-', ""));

    let asin = ASIN
        .captures(&edition_text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()));

    let cover_url = edition
        .select(&COVER)
        .next()
        .and_then(|img| {
            img.value()
                .attr("data-cover")
                .or_else(|| img.value().attr("src"))
        })
        .and_then(cover_url);

    let narrators: Vec<String> = NARRATOR
        .captures(&edition_text)
        .and_then(|cap| cap.get(1))
        .map(|m| {
            m.as_str()
                .split(',')
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let pages = PAGES
        .captures(&edition_text)
        .and_then(|cap| cap.get(1)?.as_str().parse().ok());

    let is_audiobook = !narrators.is_empty() || edition_text.to_lowercase().contains("hangoskönyv");

    Ok(ScrapedPrinting {
        publisher,
        narrators,
        publishing_year,
        published_date,
        cover_url,
        isbn,
        asin,
        pages,
        is_audiobook,
    })
}

/// Turns a cover `src` into the URL of its largest size. Editions without a cover show a
/// placeholder icon, which is not a cover.
fn cover_url(src: &str) -> Option<String> {
    let url = if src.starts_with('/') {
        format!("{BASE_URL}{src}")
    } else {
        src.to_string()
    };
    url.contains("/system/covers/")
        .then(|| url.replace("/normal/", "/big/"))
}

fn parse_hungarian_date(s: &str) -> Option<chrono::NaiveDate> {
    const MONTHS: [&str; 12] = [
        "január",
        "február",
        "március",
        "április",
        "május",
        "június",
        "július",
        "augusztus",
        "szeptember",
        "október",
        "november",
        "december",
    ];
    let cap = DATE.captures(s)?;
    let year = cap.get(1)?.as_str().parse().ok()?;
    let month_name = cap.get(2)?.as_str().to_lowercase();
    let month = MONTHS.iter().position(|m| *m == month_name)? as u32 + 1;
    let day = cap.get(3)?.as_str().parse().ok()?;
    chrono::NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hungarian_date_with_padded_day() {
        assert_eq!(
            parse_hungarian_date("Megjelenés időpontja: 2019. október  1."),
            chrono::NaiveDate::from_ymd_opt(2019, 10, 1)
        );
    }
}
