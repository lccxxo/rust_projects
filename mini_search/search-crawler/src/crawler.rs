use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};
use tracing;
use url::Url;

use crate::parser::{ParsedPage, Parser};

/// A document produced by crawling a single web page.
#[derive(Debug, Clone)]
pub struct CrawledDocument {
    pub url: String,
    pub title: String,
    pub body: String,
    pub links: Vec<String>,
    pub crawled_at: u64,
}

pub struct Crawler {
    pub max_depth: usize,
    pub concurrency: usize,
    pub delay_ms: u64,
    client: Client,
}

impl Crawler {
    pub fn new(max_depth: usize, concurrency: usize) -> Self {
        let client = Client::builder()
            .user_agent("MiniSearch/0.1.0")
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            max_depth,
            concurrency,
            delay_ms: 1000,
            client,
        }
    }

    /// Crawl starting from `seed_url`, returning all crawled documents.
    /// Crawls one depth level at a time:
    ///   - All URLs at depth N are fetched concurrently (up to `self.concurrency`).
    ///   - Links discovered at depth N become candidates for depth N+1.
    ///   - `max_depth=0` fetches only the seed URL.
    pub async fn crawl(&self, seed_url: &str) -> Vec<CrawledDocument> {
        let visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        let results: Arc<Mutex<Vec<CrawledDocument>>> = Arc::new(Mutex::new(Vec::new()));
        let sem = Arc::new(Semaphore::new(self.concurrency.max(1)));
        let domain_timers: Arc<Mutex<HashMap<String, Instant>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let mut current_level = vec![seed_url.to_string()];
        visited.lock().unwrap().insert(seed_url.to_string());

        for depth in 0..=self.max_depth {
            if current_level.is_empty() {
                break;
            }

            tracing::info!(
                "Depth {depth}: processing {} URLs (concurrency: {})",
                current_level.len(),
                self.concurrency
            );

            let mut tasks: JoinSet<Option<Vec<String>>> = JoinSet::new();
            let mut next_level_links = Vec::new();

            for url in current_level.drain(..) {
                let permit = sem.clone().acquire_owned().await.unwrap();
                let client = self.client.clone();
                let results = results.clone();
                let domain_timers = domain_timers.clone();
                let delay = Duration::from_millis(self.delay_ms);
                let max_depth = self.max_depth;

                tasks.spawn(async move {
                    let _permit = permit;

                    match fetch_and_parse_with_limit(&client, &url, &domain_timers, delay).await
                    {
                        Ok(page) => {
                            tracing::info!("[ok] {url}");

                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();

                            let doc = CrawledDocument {
                                url: url.clone(),
                                title: page.title,
                                body: page.body,
                                links: page.links.clone(),
                                crawled_at: now,
                            };

                            results.lock().unwrap().push(doc);

                            if depth < max_depth {
                                Some(page.links)
                            } else {
                                None
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to crawl {url}: {e}");
                            None
                        }
                    }
                });
            }

            // Collect links discovered at this level
            while let Some(task_result) = tasks.join_next().await {
                if let Ok(Some(links)) = task_result {
                    let mut v = visited.lock().unwrap();
                    for link in links {
                        if !v.contains(&link) {
                            v.insert(link.clone());
                            next_level_links.push(link);
                        }
                    }
                }
            }

            current_level = next_level_links;
        }

        let final_results = results.lock().unwrap().clone();
        tracing::info!(
            "Crawl finished: {} pages crawled, {} URLs discovered",
            final_results.len(),
            visited.lock().unwrap().len()
        );
        final_results
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Fetch a page, respecting per-domain rate limiting.
async fn fetch_and_parse_with_limit(
    client: &Client,
    url: &str,
    domain_timers: &Mutex<HashMap<String, Instant>>,
    min_interval: Duration,
) -> Result<ParsedPage, String> {
    // Rate-limit by domain — calculate wait first, then sleep without holding the lock
    if let Ok(parsed) = Url::parse(url) {
        if let Some(domain) = parsed.host_str() {
            let wait = {
                let mut timers = domain_timers.lock().unwrap();
                if let Some(last) = timers.get(domain) {
                    let elapsed = last.elapsed();
                    if elapsed < min_interval {
                        Some(min_interval - elapsed)
                    } else {
                        timers.insert(domain.to_string(), Instant::now());
                        None
                    }
                } else {
                    timers.insert(domain.to_string(), Instant::now());
                    None
                }
            };
            if let Some(d) = wait {
                tokio::time::sleep(d).await;
                domain_timers
                    .lock()
                    .unwrap()
                    .insert(domain.to_string(), Instant::now());
            }
        }
    }

    fetch_and_parse(client, url).await
}

async fn fetch_and_parse(client: &Client, url: &str) -> Result<ParsedPage, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.is_empty()
        && !content_type.contains("text/html")
        && !content_type.contains("text/plain")
    {
        return Err(format!("Not HTML: {content_type}"));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    Parser::parse(&html, url)
}
