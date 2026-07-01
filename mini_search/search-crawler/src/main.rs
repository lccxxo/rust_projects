use std::env;

use search_crawler::Crawler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: search-crawler <seed_url> [max_depth]");
        eprintln!("Example: search-crawler https://example.com 2");
        std::process::exit(1);
    }

    let seed_url = &args[1];
    let max_depth: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let crawler = Crawler::new(max_depth, 1);

    println!("Starting crawl at: {seed_url}");
    println!("Max depth: {max_depth}");
    println!("---");

    let docs = crawler.crawl(seed_url).await;

    println!("\n=== Crawl Results ===");
    for (i, doc) in docs.iter().enumerate() {
        println!(
            "{}. [{}] {}",
            i + 1,
            doc.url,
            doc.title
        );
        println!("   Links found: {}", doc.links.len());
        println!(
            "   Body preview: {}...",
            &doc.body.chars().take(120).collect::<String>()
        );
        println!();
    }

    println!("Total pages crawled: {}", docs.len());
}
