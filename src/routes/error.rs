use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::scraper::ScrapeError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("missing or invalid authorization")]
    Unauthorized,
    #[error("{0}")]
    BadRequest(&'static str),
    #[error(transparent)]
    Scrape(#[from] ScrapeError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Scrape(ScrapeError::NotFound) => StatusCode::NOT_FOUND,
            ApiError::Scrape(ScrapeError::Throttled { .. }) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Scrape(ScrapeError::Failed(err)) => {
                tracing::error!(error = format!("{err:#}"), "moly.hu request failed");
                StatusCode::BAD_GATEWAY
            }
        };
        let mut res = (
            status,
            Json(serde_json::json!({ "error": self.to_string() })),
        )
            .into_response();
        if let ApiError::Scrape(ScrapeError::Throttled { retry_after }) = self {
            let secs = retry_after.unwrap_or(60);
            res.headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(secs));
        }
        res
    }
}
