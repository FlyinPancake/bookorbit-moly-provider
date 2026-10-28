use std::time::{Duration, Instant};

use color_eyre::eyre;
use reqwest::{
    StatusCode,
    header::{self, HeaderMap, HeaderValue},
};

use super::{
    BASE_URL, MolySlug, ScrapeError, ScrapedBook,
    page::{parse_book_page, parse_search_page},
};

/// Spaces out requests to moly.hu so that every caller of the provider shares one budget.
struct Pacer {
    next: tokio::sync::Mutex<Instant>,
    interval: Duration,
}

impl Pacer {
    fn new(interval: Duration) -> Self {
        Self {
            next: tokio::sync::Mutex::new(Instant::now()),
            interval,
        }
    }

    async fn wait(&self) {
        let mut next = self.next.lock().await;
        tokio::time::sleep_until((*next).into()).await;
        *next = Instant::now() + self.interval;
    }
}

pub struct MolyScraper {
    client: reqwest::Client,
    pacer: Pacer,
}

impl MolyScraper {
    pub fn new(min_interval: Duration) -> Self {
        let headers = HeaderMap::from_iter([(
            header::USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/81.0",
            ),
        )]);
        Self {
            // Trust only the bundled Mozilla roots so the binary works without
            // a system CA store (e.g. in slim/distroless containers).
            client: reqwest::Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(8))
                .tls_certs_only(
                    webpki_root_certs::TLS_SERVER_ROOT_CERTS
                        .iter()
                        .map(|cert| reqwest::Certificate::from_der(cert).unwrap()),
                )
                .build()
                .unwrap(),
            pacer: Pacer::new(min_interval),
        }
    }

    async fn get(&self, url: url::Url) -> Result<(url::Url, String), ScrapeError> {
        self.pacer.wait().await;
        let res = self.client.get(url).send().await?;
        match res.status() {
            StatusCode::NOT_FOUND => return Err(ScrapeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = res
                    .headers()
                    .get(header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.trim().parse().ok());
                return Err(ScrapeError::Throttled { retry_after });
            }
            status if !status.is_success() => {
                return Err(eyre::eyre!("moly.hu answered {status}").into());
            }
            _ => {}
        }
        let final_url = res.url().clone();
        Ok((final_url, res.text().await?))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MolySlug>, ScrapeError> {
        let mut url = url::Url::parse(BASE_URL).unwrap().join("/kereses").unwrap();
        url.query_pairs_mut()
            .append_pair("utf8", "✓")
            .append_pair("query", query);
        let (final_url, body) = self.get(url).await?;
        // A search with a single exact hit (e.g. an ISBN) may redirect straight to the book.
        if let Some(slug) = final_url.path().strip_prefix("/konyvek/") {
            return Ok(vec![slug.to_string()]);
        }
        Ok(parse_search_page(&body)?)
    }

    pub async fn book(&self, slug: &str) -> Result<ScrapedBook, ScrapeError> {
        let mut url = url::Url::parse(BASE_URL).unwrap();
        url.path_segments_mut().unwrap().extend(["konyvek", slug]);
        let (final_url, body) = self.get(url).await?;
        // moly.hu redirects an unknown book to the home page instead of answering 404
        if !final_url.path().starts_with("/konyvek/") {
            return Err(ScrapeError::NotFound);
        }
        Ok(parse_book_page(&body, slug.to_string())?)
    }
}
