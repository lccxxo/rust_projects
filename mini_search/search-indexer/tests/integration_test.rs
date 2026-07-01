use std::fs;
use search_indexer::Indexer;

#[test]
fn test_indexer_create_add_search() {
    let index_dir = std::env::temp_dir().join("mini_search_test_index");
    fs::remove_dir_all(&index_dir).ok();

    // Create indexer
    let mut indexer = Indexer::open_or_create(&index_dir).expect("open_or_create");

    // Add documents
    indexer
        .add_document(
            "https://example.com/rust",
            "Rust 编程语言",
            "Rust 是一门系统编程语言，具有内存安全和高性能的特点。",
            1710000000,
        )
        .expect("add_document 1");

    indexer
        .add_document(
            "https://example.com/python",
            "Python 入门教程",
            "Python 是一门易于学习的动态语言，广泛应用于数据科学。",
            1710000001,
        )
        .expect("add_document 2");

    indexer
        .add_document(
            "https://example.com/go",
            "Go 语言并发编程",
            "Go 语言由 Google 开发，原生支持并发编程。Rust 和 Go 都是现代系统语言。",
            1710000002,
        )
        .expect("add_document 3");

    // Commit
    indexer.commit().expect("commit");

    // Search
    let reader = indexer.reader().expect("reader");
    let searcher = reader.searcher();

    let schema = indexer.schema();
    let _title_field = schema.get_field("title").unwrap();
    let body_field = schema.get_field("body").unwrap();

    // Parse query and search body field
    use tantivy::{
        query::QueryParser,
        collector::TopDocs,
        schema::Value,
    };

    let query_parser = QueryParser::for_index(indexer.index(), vec![body_field]);
    let query = query_parser.parse_query("Rust").expect("parse query");

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(10))
        .expect("search");

    assert_eq!(top_docs.len(), 2, "Should find 2 docs mentioning Rust");

    // Both docs match "Rust"; verify they're the right URLs
    let mut urls: Vec<String> = top_docs
        .iter()
        .map(|(_score, addr)| {
            let doc: tantivy::TantivyDocument = searcher.doc(*addr).unwrap();
            doc.get_first(schema.get_field("url").unwrap())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    urls.sort();
    assert_eq!(urls[0], "https://example.com/go");
    assert_eq!(urls[1], "https://example.com/rust");

    // Cleanup
    drop(indexer);
    fs::remove_dir_all(&index_dir).ok();
}
