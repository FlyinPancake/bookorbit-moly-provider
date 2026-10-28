use axum::{
    Json,
    extract::{Query, State},
};
use futures::StreamExt;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{AppState, error::ApiError};
use crate::scraper::ScrapeError;
use crate::{dtos::MediaType, mapping};

const DEFAULT_LIMIT: usize = 5;
/// Every match costs a page fetch, so keep this low enough to answer within BookOrbit's timeout
const MAX_LIMIT: usize = 10;
const CONCURRENT_FETCHES: usize = 4;

pub(super) fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(search))
}

#[derive(utoipa::IntoParams, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query, rename_all = "camelCase")]
pub struct SearchQuery {
    /// Title of the book
    query: Option<String>,
    author: Option<String>,
    isbn: Option<String>,
    /// Maximum number of matches to return
    limit: Option<usize>,
    /// The kind of book being matched
    #[param(inline)]
    media_type: Option<MediaType>,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub matches: Vec<crate::dtos::BookResult>,
    /// Set when some matches could not be fetched
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
}

fn non_blank(s: &Option<String>) -> Option<&str> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

#[utoipa::path(
    get,
    path = "/search",
    params(SearchQuery),
    responses(
        (status = OK, body = SearchResults),
        (status = BAD_REQUEST, description = "Neither a query nor an ISBN was given"),
        (status = TOO_MANY_REQUESTS, description = "moly.hu is throttling requests"),
        (status = BAD_GATEWAY, description = "moly.hu could not be reached or parsed"),
    )
)]
/// Search for books
///
/// Returns the books matching a title, author or ISBN, best first.
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResults>, ApiError> {
    let isbn: Option<String> =
        non_blank(&params.isbn).map(|i| i.chars().filter(char::is_ascii_alphanumeric).collect());
    let title = non_blank(&params.query);
    if isbn.is_none() && title.is_none() {
        return Err(ApiError::BadRequest("either query or isbn is required"));
    }
    let media = params.media_type.unwrap_or(MediaType::Book);
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    let mut slugs = match &isbn {
        Some(isbn) => state.scraper.search(isbn).await?,
        None => vec![],
    };
    if slugs.is_empty()
        && let Some(title) = title
    {
        let term = match non_blank(&params.author) {
            Some(author) => format!("{title} {author}"),
            None => title.to_string(),
        };
        slugs = state.scraper.search(&term).await?;
    }
    let mut seen = std::collections::HashSet::new();
    slugs.retain(|slug| seen.insert(slug.clone()));
    slugs.truncate(limit);

    // `buffered` keeps moly's ranking
    let fetched: Vec<_> = futures::stream::iter(slugs)
        .map(|slug| {
            let scraper = state.scraper.clone();
            async move { scraper.book(&slug).await }
        })
        .buffered(CONCURRENT_FETCHES)
        .collect()
        .await;

    let mut matches = Vec::with_capacity(fetched.len());
    let mut first_error = None;
    for result in fetched {
        match result {
            Ok(book) => matches.push(mapping::to_book_result(&book, media, isbn.as_deref())),
            // the search page linked a book that is gone, nothing is missing
            Err(ScrapeError::NotFound) => {}
            Err(err) => {
                tracing::warn!(error = %err, "failed to fetch a search match");
                first_error.get_or_insert(err);
            }
        }
    }

    match first_error {
        Some(err) if matches.is_empty() => Err(err.into()),
        partial => Ok(Json(SearchResults {
            matches,
            partial: partial.is_some(),
        })),
    }
}
