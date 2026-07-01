use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use serde::Deserialize;
use search_indexer::Indexer;

/// JSON input format matching the crawler's output.
#[derive(Deserialize)]
struct InputDocument {
    url: String,
    title: String,
    body: String,
    crawled_at: u64,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: search-indexer <index_dir> [input_file]");
        eprintln!("  input_file: JSON lines file (one doc per line). Reads stdin if omitted.");
        std::process::exit(1);
    }

    let index_dir = PathBuf::from(&args[1]);
    let input_file = args.get(2);

    let mut indexer = Indexer::open_or_create(&index_dir).expect("Failed to open index");

    let reader: Box<dyn BufRead> = match input_file {
        Some(path) => {
            let file = File::open(path).expect("Failed to open input file");
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(io::stdin())),
    };

    let mut count = 0;
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<InputDocument>(&line) {
            Ok(doc) => {
                if let Err(e) = indexer.add_document(&doc.url, &doc.title, &doc.body, doc.crawled_at) {
                    eprintln!("Failed to index {}: {e}", doc.url);
                } else {
                    count += 1;
                    if count % 100 == 0 {
                        println!("Indexed {count} documents...");
                    }
                }
            }
            Err(e) => {
                eprintln!("Skipping invalid JSON line: {e}");
            }
        }
    }

    let opstamp = indexer.commit().expect("Failed to commit index");
    println!("Done. {count} documents indexed, opstamp: {opstamp}");
}
