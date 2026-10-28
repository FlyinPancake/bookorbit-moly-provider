//! Turns scraped moly.hu pages into BookOrbit records

use rust_decimal::Decimal;

use crate::dtos::{BookResult, Description, LanguageCode, MediaType, SeriesEntry};
use crate::scraper::{ScrapedBook, ScrapedPrinting};

/// Appended to a slug to address the audiobook edition of a book.
/// It ends up as a URL fragment in the book URL template, so the link still works.
pub const AUDIOBOOK_SUFFIX: &str = "#audiobook";

/// Splits a provider ID into the moly.hu slug and the media type it addresses
pub fn parse_id(id: &str) -> Option<(&str, MediaType)> {
    let (slug, media) = match id.strip_suffix(AUDIOBOOK_SUFFIX) {
        Some(slug) => (slug, MediaType::Audiobook),
        None => (id, MediaType::Book),
    };
    let valid = !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    valid.then_some((slug, media))
}

fn pick_printing<'a>(
    book: &'a ScrapedBook,
    media: MediaType,
    isbn: Option<&str>,
) -> Option<&'a ScrapedPrinting> {
    let printings = &book.printings;
    // editions are listed newest first
    isbn.and_then(|isbn| printings.iter().find(|p| p.isbn.as_deref() == Some(isbn)))
        .or_else(|| match media {
            MediaType::Audiobook => printings.iter().find(|p| p.is_audiobook),
            _ => printings
                .iter()
                .find(|p| !p.is_audiobook && p.cover_url.is_some())
                .or_else(|| printings.iter().find(|p| !p.is_audiobook)),
        })
        .or_else(|| printings.first())
}

fn non_empty<T>(v: Vec<T>) -> Option<Vec<T>> {
    (!v.is_empty()).then_some(v)
}

/// Builds the record for `book`, preferring the edition that matches `media` and `isbn`.
pub fn to_book_result(book: &ScrapedBook, media: MediaType, isbn: Option<&str>) -> BookResult {
    let printing = pick_printing(book, media, isbn);
    let is_audiobook = printing.is_some_and(|p| p.is_audiobook);
    let provider_id = if is_audiobook {
        format!("{}{AUDIOBOOK_SUFFIX}", book.slug)
    } else {
        book.slug.clone()
    };
    let url = crate::consts::PROVIDER_BOOK_TEMPLATE_URL.replace("{id}", &provider_id);

    let isbn = printing.and_then(|p| p.isbn.as_deref());
    let narrators = printing.and_then(|p| non_empty(p.narrators.clone()));
    let series = book
        .series
        .iter()
        .map(|s| SeriesEntry {
            series: s.name.clone(),
            sequence: s.idx.map(Decimal::from),
        })
        .collect();

    BookResult {
        provider_id,
        title: book.title.clone(),
        media_type: Some(if is_audiobook {
            MediaType::Audiobook
        } else {
            MediaType::Book
        }),
        subtitle: book.subtitle.clone(),
        authors: non_empty(book.authors.clone()),
        narrators,
        description: book.description.as_deref().map(Description::new),
        publisher: printing.map(|p| p.publisher.clone()),
        published_year: printing.map(|p| p.publishing_year.clone()),
        published_date: printing.and_then(|p| p.published_date),
        language_code: book.language.map(|code| LanguageCode(code.to_string())),
        isbn: None,
        isbn13: isbn.filter(|i| i.len() == 13).map(str::to_string),
        isbn10: isbn.filter(|i| i.len() == 10).map(str::to_string),
        pages: printing.and_then(|p| p.pages).filter(|_| !is_audiobook),
        duration: None,
        genres: non_empty(book.tags.clone()),
        series: non_empty(series),
        cover: printing
            .and_then(|p| p.cover_url.as_deref())
            .and_then(|c| url::Url::parse(c).ok()),
        cover_width: None,
        cover_height: None,
        rating: book.rating,
        rating_count: book.rating.and(book.rating_count),
        url: url::Url::parse(&url).ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraper::{ScrapedPrinting, ScrapedSeries};

    fn printing(isbn: &str, narrator: Option<&str>) -> ScrapedPrinting {
        ScrapedPrinting {
            publishing_year: "2020".into(),
            published_date: None,
            publisher: "Agave".into(),
            cover_url: Some("https://moly.hu/system/covers/big/covers_1.jpg".into()),
            narrators: narrator.into_iter().map(Into::into).collect(),
            isbn: Some(isbn.into()),
            asin: None,
            pages: Some(300),
            is_audiobook: narrator.is_some(),
        }
    }

    fn book() -> ScrapedBook {
        ScrapedBook {
            slug: "frank-herbert-dune".into(),
            title: "Dune".into(),
            series: vec![ScrapedSeries {
                name: "Dune".into(),
                idx: Some(1),
            }],
            subtitle: None,
            description: Some("Arrakis".into()),
            authors: vec!["Frank Herbert".into()],
            tags: vec!["sci-fi".into()],
            language: Some("hu"),
            rating: Some(4.5),
            rating_count: Some(10),
            printings: vec![
                printing("9789634190000", Some("Kristóf Tibor")),
                printing("9789634191111", None),
                printing("9789634192222", None),
            ],
        }
    }

    #[test]
    fn book_search_skips_audiobook_editions() {
        let result = to_book_result(&book(), MediaType::Book, None);

        assert_eq!(result.provider_id, "frank-herbert-dune");
        assert_eq!(result.media_type, Some(MediaType::Book));
        assert_eq!(result.isbn13.as_deref(), Some("9789634191111"));
        assert_eq!(result.narrators, None);
        assert_eq!(
            result.url.unwrap().as_str(),
            "https://moly.hu/konyvek/frank-herbert-dune"
        );
    }

    #[test]
    fn audiobook_search_picks_narrated_edition() {
        let result = to_book_result(&book(), MediaType::Audiobook, None);

        assert_eq!(result.provider_id, "frank-herbert-dune#audiobook");
        assert_eq!(result.media_type, Some(MediaType::Audiobook));
        assert_eq!(result.narrators, Some(vec!["Kristóf Tibor".to_string()]));
        assert_eq!(result.pages, None);
    }

    #[test]
    fn isbn_selects_matching_edition() {
        let result = to_book_result(&book(), MediaType::Book, Some("9789634192222"));

        assert_eq!(result.isbn13.as_deref(), Some("9789634192222"));
    }

    #[test]
    fn series_sequence_serializes_as_string() {
        let result = to_book_result(&book(), MediaType::Book, None);
        let json = serde_json::to_value(&result).unwrap();

        assert_eq!(json["series"][0]["sequence"], "1");
        assert_eq!(json["languageCode"], "hu");
        assert_eq!(json["providerId"], "frank-herbert-dune");
    }

    #[test]
    fn missing_fields_are_left_out() {
        let mut book = book();
        book.printings.clear();
        book.series.clear();
        let json = serde_json::to_value(to_book_result(&book, MediaType::Book, None)).unwrap();

        // the conformance checker rejects `"narrators": null` and the like
        assert!(json.as_object().unwrap().values().all(|v| !v.is_null()));
        assert!(json.get("narrators").is_none());
        assert!(json.get("cover").is_none());
    }

    #[test]
    fn ids_round_trip() {
        assert_eq!(
            parse_id("frank-herbert-dune#audiobook"),
            Some(("frank-herbert-dune", MediaType::Audiobook))
        );
        assert_eq!(
            parse_id("frank-herbert-dune"),
            Some(("frank-herbert-dune", MediaType::Book))
        );
        assert_eq!(parse_id("../etc"), None);
        assert_eq!(parse_id(""), None);
    }
}
