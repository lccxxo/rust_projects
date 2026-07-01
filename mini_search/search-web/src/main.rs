use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{routing::{get, post}, Router};
use tower_http::services::ServeDir;

mod handlers;

use handlers::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Default index path; can be overridden via env var
    let index_path = std::env::var("MINISEARCH_INDEX")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data/index"));

    // Auto-create an empty index if none exists yet
    if !index_path.exists() {
        let mut indexer = search_indexer::Indexer::open_or_create(&index_path)
            .expect("Failed to create index");
        indexer.commit().ok();
        drop(indexer);
        tracing::info!("Created empty index at {:?}", index_path);
    }

    let engine = search_engine::QueryEngine::open(&index_path)
        .expect("Failed to open index");

    let state = Arc::new(AppState {
        index_path: index_path.clone(),
        engine,
        crawl_status: Arc::new(Mutex::new(Default::default())),
    });

    let app = Router::new()
        .route("/api/search", get(handlers::search_handler))
        .route("/api/crawl", post(handlers::crawl_handler))
        .route("/api/status", get(handlers::status_handler))
        .fallback_service(ServeDir::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/static"),
        ))
        .with_state(state);

    let addr = "127.0.0.1:3000";
    tracing::info!("MiniSearch web server starting at http://{addr}");
    println!("MiniSearch ready at http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
