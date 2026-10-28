//! Scrapes book data from moly.hu. Ported from the `mscraper` crate of moly-hu-api.

/// A `Selector` parsed once, on first use
macro_rules! static_selector {
    ($name:ident, $s:expr) => {
        static $name: std::sync::LazyLock<::scraper::Selector> = std::sync::LazyLock::new(|| {
            ::scraper::Selector::parse($s)
                .unwrap_or_else(|e| panic!("invalid selector {:?}: {e}", $s))
        });
    };
}

/// A `Regex` compiled once, on first use
macro_rules! static_regex {
    ($name:ident, $s:expr) => {
        static $name: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new($s).unwrap());
    };
}

mod client;
mod edition;
mod language;
mod page;

use color_eyre::eyre;

pub use client::MolyScraper;

const BASE_URL: &str = "https://moly.hu";

#[derive(Debug, thiserror::Error)]
pub enum ScrapeError {
    #[error("not found on moly.hu")]
    NotFound,
    #[error("moly.hu is throttling requests")]
    Throttled { retry_after: Option<u64> },
    #[error(transparent)]
    Failed(#[from] eyre::Report),
}

impl From<reqwest::Error> for ScrapeError {
    fn from(err: reqwest::Error) -> Self {
        Self::Failed(err.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrapedBook {
    pub slug: String,
    pub title: String,
    pub series: Vec<ScrapedSeries>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    /// Tags, without the ones that name the book's language
    pub tags: Vec<String>,
    /// ISO 639-1 code of the book's language
    pub language: Option<&'static str>,
    /// Community rating on a 0-5 scale
    pub rating: Option<f64>,
    pub rating_count: Option<u32>,
    pub printings: Vec<ScrapedPrinting>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrapedPrinting {
    pub publishing_year: String,
    pub published_date: Option<chrono::NaiveDate>,
    pub publisher: String,
    pub cover_url: Option<String>,
    pub narrators: Vec<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub pages: Option<u32>,
    pub is_audiobook: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrapedSeries {
    pub name: String,
    pub idx: Option<i32>,
}

pub type MolySlug = String;
