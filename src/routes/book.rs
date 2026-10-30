use axum::{
    Json,
    extract::{Path, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{AppState, error::ApiError};
use crate::scraper::ScrapeError;
use crate::{dtos::BookResult, mapping};

pub(super) fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(lookup))
}

#[utoipa::path(
    get,
    path = "/book/{id}",
    params(("id" = String, Path, description = "A `providerId` returned by search")),
    responses(
        (status = OK, body = BookResult),
        (status = NOT_FOUND, description = "No book with this ID"),
        (status = TOO_MANY_REQUESTS, description = "moly.hu is throttling requests"),
        (status = BAD_GATEWAY, description = "moly.hu could not be reached or parsed"),
    )
)]
/// Look up a book by ID
pub async fn lookup(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<BookResult>, ApiError> {
    let (slug, media, isbn) = mapping::parse_id(&id).ok_or(ScrapeError::NotFound)?;
    let book = state.scraper.book(slug).await?;
    Ok(Json(mapping::to_book_result(&book, media, isbn.as_deref())))
}
