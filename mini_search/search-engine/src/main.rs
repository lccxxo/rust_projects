use std::env;

use search_engine::QueryEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: search-engine <index_dir> <query> [page]");
        eprintln!("Example: search-engine ./data/index \"Rust 编程\" 1");
        std::process::exit(1);
    }

    let index_dir = &args[1];
    let query_str = &args[2];
    let page: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);

    let engine = QueryEngine::open(std::path::Path::new(index_dir)).expect("Failed to open index");

    let doc_count = engine.doc_count().expect("Failed to get doc count");
    println!("Index: {index_dir} ({doc_count} documents)");
    println!("Query: \"{query_str}\"");
    println!("---");

    match engine.search(query_str, page, 10) {
        Ok(response) => {
            println!(
                "Found {} results (page {}/{})",
                response.total,
                response.page,
                (response.total + response.page_size - 1) / response.page_size
            );
            println!();

            for (i, result) in response.results.iter().enumerate() {
                let rank = (page - 1) * 10 + i + 1;
                println!("{rank}. {}", result.title);
                println!("   URL:   {}", result.url);
                println!("   Score: {:.4}", result.score);
                println!("   {}...", result.snippet);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Search failed: {e}");
            std::process::exit(1);
        }
    }
}
