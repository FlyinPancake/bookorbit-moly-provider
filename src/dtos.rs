use rust_decimal::Decimal;

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Book,
    Audiobook,
    Comic,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Debug)]
#[serde(transparent)]
/// A description
///
/// At most 50,000 characters
pub struct Description(String);

impl Description {
    const MAX_CHARS: usize = 50_000;

    pub fn new(s: &str) -> Self {
        Self(s.chars().take(Self::MAX_CHARS).collect())
    }
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Debug)]
#[serde(transparent)]
/// Two letter language code, e.g. "en" or "hu"
pub struct LanguageCode(pub String);

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SeriesEntry {
    /// name of the series
    pub series: String,
    /// index of the book in the series, e.g. 1 for the first book
    pub sequence: Option<Decimal>,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BookResult {
    pub provider_id: String,
    pub title: String,
    pub media_type: Option<MediaType>,
    pub subtitle: Option<String>,
    pub authors: Option<Vec<String>>,
    pub narrators: Option<Vec<String>>,
    pub description: Option<Description>,
    pub publisher: Option<String>,
    pub published_year: Option<String>,
    pub published_date: Option<chrono::NaiveDate>,
    pub language_code: Option<LanguageCode>,
    pub isbn: Option<String>,
    pub isbn13: Option<String>,
    pub isbn10: Option<String>,
    pub pages: Option<u32>,
    /// Audiobook duration in seconds
    pub duration: Option<u32>,
    pub genres: Option<Vec<String>>,
    pub series: Option<Vec<SeriesEntry>>,
    pub cover: Option<url::Url>,
    /// cover width in pixels
    pub cover_width: Option<u32>,
    /// cover height in pixels
    pub cover_height: Option<u32>,
    /// rating from 0 to 5
    pub rating: Option<f64>,
    pub rating_count: Option<u32>,
    pub url: Option<url::Url>,
}
