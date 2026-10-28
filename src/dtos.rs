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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<Decimal>,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BookResult {
    pub provider_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrators: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<chrono::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<LanguageCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn10: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<u32>,
    /// Audiobook duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<Vec<SeriesEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<url::Url>,
    /// cover width in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_width: Option<u32>,
    /// cover height in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_height: Option<u32>,
    /// rating from 0 to 5
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<url::Url>,
}
