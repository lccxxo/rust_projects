use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use search_crawler::Crawler;
use search_engine::{QueryEngine, SearchResponse};
use search_indexer::Indexer;

// ── Crawl Status ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CrawledPageInfo {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrawlStatus {
    pub running: bool,
    pub url: String,
    pub pages_crawled: usize,
    pub pages_indexed: usize,
    pub pages: Vec<CrawledPageInfo>,
    pub error: Option<String>,
}

impl Default for CrawlStatus {
    fn default() -> Self {
        Self {
            running: false,
            url: String::new(),
            pages_crawled: 0,
            pages_indexed: 0,
            pages: Vec::new(),
            error: None,
        }
    }
}

// ── App State ──────────────────────────────────────────────────

pub struct AppState {
    pub index_path: PathBuf,
    pub engine: QueryEngine,
    pub crawl_status: Arc<Mutex<CrawlStatus>>,
}

// ── Query params ───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub page: Option<usize>,
}

// ── Request bodies ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CrawlRequest {
    pub url: String,
    pub max_depth: Option<usize>,
}

// ── Response types ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct StatusResponse {
    pub index_path: String,
    pub doc_count: u64,
    pub crawl: CrawlStatus,
}

#[derive(Serialize)]
pub struct CrawlStartedResponse {
    pub status: String,
    pub url: String,
    pub max_depth: usize,
}

// ── Handlers ───────────────────────────────────────────────────

/// GET /api/search?q=Rust&page=1
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Json<SearchResponse> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = 10;

    match state.engine.search(&params.q, page, page_size) {
        Ok(response) => Json(response),
        Err(e) => {
            tracing::error!("Search error: {e}");
            Json(SearchResponse {
                query: params.q,
                total: 0,
                page,
                page_size,
                results: vec![],
            })
        }
    }
}

/// POST /api/crawl  —  spawns a background crawl + index task.
pub async fn crawl_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CrawlRequest>,
) -> Json<CrawlStartedResponse> {
    let max_depth = body.max_depth.unwrap_or(0);
    let index_path = state.index_path.clone();
    let url = body.url.clone();
    let status = state.crawl_status.clone();

    // Mark as running
    {
        let mut s = status.lock().unwrap();
        s.running = true;
        s.url = url.clone();
        s.pages_crawled = 0;
        s.pages_indexed = 0;
        s.error = None;
    }

    tokio::spawn(async move {
        tracing::info!("Crawl started: url={url}, max_depth={max_depth}");

        let crawler = Crawler::new(max_depth, 5);
        let docs = crawler.crawl(&url).await;
        let crawled = docs.len();

        let crawled_pages: Vec<CrawledPageInfo> = docs
            .iter()
            .map(|d| CrawledPageInfo {
                url: d.url.clone(),
                title: d.title.clone(),
            })
            .collect();

        {
            let mut s = status.lock().unwrap();
            s.pages_crawled = crawled;
            s.pages = crawled_pages;
        }

        tracing::info!("Crawl finished: {crawled} documents");

        if docs.is_empty() {
            let mut s = status.lock().unwrap();
            s.running = false;
            s.error = Some("No pages crawled — the site may be unreachable or not HTML.".into());
            return;
        }

        match Indexer::open_or_create(&index_path) {
            Ok(mut indexer) => {
                let mut count = 0;
                for doc in &docs {
                    if indexer
                        .add_document(&doc.url, &doc.title, &doc.body, doc.crawled_at)
                        .is_ok()
                    {
                        count += 1;
                    }
                }
                match indexer.commit() {
                    Ok(_) => {
                        tracing::info!("Indexed {count} documents");
                        let mut s = status.lock().unwrap();
                        s.running = false;
                        s.pages_indexed = count;
                    }
                    Err(e) => {
                        let mut s = status.lock().unwrap();
                        s.running = false;
                        s.error = Some(format!("Index commit failed: {e}"));
                    }
                }
            }
            Err(e) => {
                let mut s = status.lock().unwrap();
                s.running = false;
                s.error = Some(format!("Failed to open index: {e}"));
            }
        }
    });

    Json(CrawlStartedResponse {
        status: "started".to_string(),
        url: body.url.clone(),
        max_depth,
    })
}

/// GET /api/status  —  includes crawl progress.
pub async fn status_handler(
    State(state): State<Arc<AppState>>,
) -> Json<StatusResponse> {
    let doc_count = state.engine.doc_count().unwrap_or(0);
    let crawl = state.crawl_status.lock().unwrap().clone();

    Json(StatusResponse {
        index_path: state.index_path.to_string_lossy().to_string(),
        doc_count,
        crawl,
    })
}
