use axum::{Json, response::IntoResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::AppState;
use crate::dtos::MediaType;

pub(super) fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(info))
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportedField {
    Title,
    Subtitle,
    Description,
    Cover,
    AudioCover,
    Authors,
    Publisher,
    Isbn,
    PublishedYear,
    Language,
    PageCount,
    CommunityRating,
    SeriesName,
    SeriesIndex,
    Genres,
    Narrators,
    Duration,
    Abridged,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchParam {
    Isbn,
    Limit,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoverShape {
    Portrait,
    Square,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    /// The maximum number of concurrent requests allowed to the provider (1-4)
    max_concurrency: u32,
    /// The minimum interval between requests to the provider, in milliseconds (0-60000)
    min_interval_ms: u32,
}

// need to make some validation magic here
#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
/// A color in hex format, e.g. "#ff0000"
pub struct Color(String);

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
/// A URL with an `{id}` placeholder that points to a book on the provider's website, e.g. "https://example.com/book/{id}"
pub struct BookUrlTemplate(String);

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
/// The max length of the ID that can be used to look up a book on the provider's website
/// At most 512 characters
pub struct IdMaxLength(u32);

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    name: String,
    key: String,
    version: String,
    /// Optional URL to the provider's avatar image
    avatar_url: Option<url::Url>,
    /// Brand color of the provider, in hex format, e.g. "#ff0000"
    color: Option<Color>,
    /// The media types supported by the provider
    media_types: Option<Vec<MediaType>>,
    /// Whether the provider supports lookup of books by ID
    lookup: bool,
    search_params: Option<Vec<SearchParam>>,
    supported_fields: Option<Vec<SupportedField>>,
    book_url_template: Option<BookUrlTemplate>,
    id_max_length: Option<IdMaxLength>,
    rate_limit: Option<RateLimit>,
    /// If the only responds with one cover shape, this field will be set to that shape.
    cover_shape: Option<CoverShape>,
    authentication_required: bool,
}

impl ProviderInfo {
    fn new(authentication_required: bool) -> Self {
        Self {
            name: crate::consts::PROVIDER_NAME.to_string(),
            version: crate::consts::PROVIDER_VERSION.to_string(),
            authentication_required,
            key: crate::consts::PROVIDER_ID.to_string(),
            avatar_url: Some(
                url::Url::parse(crate::consts::PROVIDER_FAVICON_URL)
                    .expect("const PROVIDER_FAVICON_URL is not a valid URL"),
            ),
            color: Some(Color(crate::consts::PROVIDER_COLOR.to_string())),
            media_types: Some(vec![MediaType::Book, MediaType::Audiobook]),
            lookup: true,
            search_params: Some(vec![SearchParam::Isbn, SearchParam::Limit]),
            // only what moly.hu pages actually carry
            supported_fields: Some(vec![
                SupportedField::Title,
                SupportedField::Subtitle,
                SupportedField::Description,
                SupportedField::Cover,
                SupportedField::AudioCover,
                SupportedField::Authors,
                SupportedField::Publisher,
                SupportedField::Isbn,
                SupportedField::PublishedYear,
                SupportedField::Language,
                SupportedField::PageCount,
                SupportedField::CommunityRating,
                SupportedField::SeriesName,
                SupportedField::SeriesIndex,
                SupportedField::Genres,
                SupportedField::Narrators,
            ]),
            book_url_template: Some(BookUrlTemplate(
                crate::consts::PROVIDER_BOOK_TEMPLATE_URL.to_string(),
            )),
            // Moly uses really long IDs for books, so we need to allow for that.
            // The longest ID allowed is 512 characters, so we'll use that as the max length.
            id_max_length: Some(IdMaxLength(512)),
            // pretty conservative rate limits, since we scrape
            rate_limit: Some(RateLimit {
                max_concurrency: 1,
                min_interval_ms: 2000,
            }),
            // Moly has both portrait and square covers, so we don't want to limit the shape
            cover_shape: None,
        }
    }
}

#[utoipa::path(
    get,
    path = "/info",
    responses(
        (status = OK, body = ProviderInfo)
    )
)]
/// Get provider info
///
/// Returns the name, version, authentication methods and other information about the provider.
pub async fn info(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    Json(ProviderInfo::new(state.auth_token.is_some()))
}
