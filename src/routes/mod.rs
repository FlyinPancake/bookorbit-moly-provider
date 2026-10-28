mod auth;
mod book;
mod error;
mod provider;
mod search;

use std::sync::Arc;

use axum::middleware;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::scraper::MolyScraper;

#[derive(utoipa::OpenApi)]
#[openapi(info(
    title = "BookOrbit Moly Provider",
    description = "API for the BookOrbit Moly Provider",
))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub scraper: Arc<MolyScraper>,
    /// When set, every API request must carry it in the `Authorization` header
    pub auth_token: Option<Arc<str>>,
}

pub fn router(state: AppState) -> axum::Router {
    let router = utoipa_axum::router::OpenApiRouter::new()
        .nest("/provider", provider::router())
        .merge(search::router())
        .merge(book::router());

    let (axum_router, openapi) = router.split_for_parts();
    let mut openapi = ApiDoc::openapi().merge_from(openapi);
    if state.auth_token.is_some() {
        auth::document_auth(&mut openapi);
    }
    axum_router
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
        .with_state(state)
        .merge(Scalar::with_url("/docs", openapi))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{
        body::Body,
        extract::Request,
        http::{StatusCode, header},
        response::Response,
    };
    use tower::ServiceExt;

    use super::*;

    fn app(auth_token: Option<&str>) -> axum::Router {
        router(AppState {
            scraper: Arc::new(MolyScraper::new(Duration::ZERO)),
            auth_token: auth_token.map(Into::into),
        })
    }

    async fn get(app: axum::Router, uri: &str, auth: Option<&str>) -> Response {
        let mut req = Request::get(uri);
        if let Some(auth) = auth {
            req = req.header(header::AUTHORIZATION, auth);
        }
        app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap()
    }

    #[tokio::test]
    async fn info_is_open_without_token() {
        let res = get(app(None), "/provider/info", None).await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn token_is_enforced() {
        let res = get(app(Some("secret")), "/provider/info", None).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let res = get(app(Some("secret")), "/provider/info", Some("Bearer wrong")).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let res = get(app(Some("secret")), "/provider/info", Some("Bearer secret")).await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = get(app(Some("secret")), "/provider/info", Some("secret")).await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn docs_stay_open() {
        let res = get(app(Some("secret")), "/docs", None).await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    async fn docs_body(auth_token: Option<&str>) -> String {
        let res = get(app(auth_token), "/docs", None).await;
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn docs_declare_auth_only_when_token_is_set() {
        let docs = docs_body(Some("secret")).await;
        assert!(
            docs.contains(r#""securitySchemes":{"authToken":{"type":"http","scheme":"bearer""#)
        );
        assert!(docs.contains(r#""security":[{"authToken":[]}]"#));

        assert!(!docs_body(None).await.contains("authToken"));
    }

    #[tokio::test]
    async fn search_without_query_or_isbn_is_rejected() {
        let res = get(app(None), "/search?author=Herbert", None).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invalid_id_is_not_found() {
        let res = get(app(None), "/book/..%2Fsecret", None).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
