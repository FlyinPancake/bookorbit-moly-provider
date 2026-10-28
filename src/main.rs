pub(crate) mod consts;
mod dtos;
mod mapping;
mod routes;
mod scraper;

use std::{sync::Arc, time::Duration};

use color_eyre::eyre::WrapErr;

/// Default gap between two requests to moly.hu
const DEFAULT_MIN_INTERVAL_MS: u64 = 500;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let port: u16 = env_or("PORT", 8080)?;
    let min_interval =
        Duration::from_millis(env_or("MOLY_MIN_INTERVAL_MS", DEFAULT_MIN_INTERVAL_MS)?);
    let auth_token = std::env::var("AUTH_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
        .map(|t| Arc::from(t.trim()));
    if auth_token.is_none() {
        tracing::warn!("AUTH_TOKEN is not set, the API is open to anyone who can reach it");
    }

    let router = routes::router(routes::AppState {
        scraper: Arc::new(scraper::MolyScraper::new(min_interval)),
        auth_token,
    });

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Waits for Ctrl+C or SIGTERM. In a container the binary is PID 1, which gets no default
/// SIGTERM handler, so without this `docker stop` waits out its timeout.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
    };
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("Shutting down");
}

fn env_or<T: std::str::FromStr>(name: &str, default: T) -> color_eyre::Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(name) {
        Ok(v) => v.parse().wrap_err_with(|| format!("invalid {name}")),
        Err(_) => Ok(default),
    }
}
