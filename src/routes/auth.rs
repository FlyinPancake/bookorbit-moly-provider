use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::{AppState, error::ApiError};

const AUTH_SCHEME: &str = "authToken";

/// Declares the `AUTH_TOKEN` bearer scheme and requires it on every operation. Only called when
/// a token is set, so the docs of an open instance don't ask for one.
pub(super) fn document_auth(openapi: &mut utoipa::openapi::OpenApi) {
    use utoipa::openapi::security::{
        HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme,
    };

    openapi
        .components
        .get_or_insert_with(Default::default)
        .add_security_scheme(
            AUTH_SCHEME,
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .description(Some("The provider's `AUTH_TOKEN`"))
                    .build(),
            ),
        );
    openapi.security = Some(vec![SecurityRequirement::new::<_, _, String>(
        AUTH_SCHEME,
        [],
    )]);
}

/// Accepts either `Bearer <token>` or the bare token, since BookOrbit sends the header as entered.
pub(super) async fn authorize(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let Some(token) = state.auth_token.as_deref() else {
        return next.run(req).await;
    };
    let given = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v).trim());
    if given == Some(token) {
        next.run(req).await
    } else {
        ApiError::Unauthorized.into_response()
    }
}
