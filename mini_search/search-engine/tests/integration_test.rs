use std::fs;

use search_engine::QueryEngine;
use search_indexer::Indexer;

#[test]
fn test_search_with_index() {
    let index_dir = std::env::temp_dir().join("mini_search_engine_test");
    fs::remove_dir_all(&index_dir).ok();

    // Build an index
    let mut indexer = Indexer::open_or_create(&index_dir).expect("open_or_create");

    indexer
        .add_document(
            "https://example.com/rust-book",
            "Rust 程序设计语言",
            "Rust 是一门系统编程语言，专注于内存安全和高性能。它由 Mozilla 开发。",
            1710000000,
        )
        .expect("add_doc 1");

    indexer
        .add_document(
            "https://example.com/python",
            "Python 入门教程",
            "Python 是一种解释型语言，易于学习，适合数据科学和 Web 开发。",
            1710000001,
        )
        .expect("add_doc 2");

    indexer
        .add_document(
            "https://example.com/go",
            "Go 并发编程指南",
            "Go 语言由 Google 开发，原生支持并发。与 Rust 一样，Go 也适合系统编程。",
            1710000002,
        )
        .expect("add_doc 3");

    indexer.commit().expect("commit");
    drop(indexer);

    // Open the index with the search engine
    let engine = QueryEngine::open(&index_dir).expect("open engine");

    // Search for "Rust"
    let response = engine.search("Rust", 1, 10).expect("search");

    assert_eq!(response.query, "Rust");
    assert_eq!(response.page, 1);
    assert_eq!(response.total, 2, "Should find 2 docs mentioning Rust");

    // First result should be the Rust book (title match boosted)
    assert!(
        response.results[0].title.contains("Rust"),
        "Rust book should rank first due to title boost"
    );

    // Snippet should contain "Rust"
    assert!(
        response.results[0].snippet.contains("Rust"),
        "Snippet should contain the keyword"
    );

    // Test pagination with a term that matches multiple docs
    let page1 = engine.search("编程", 1, 2).expect("search page 1");
    assert_eq!(page1.results.len(), 2);
    assert!(page1.total >= 2);

    let page2 = engine.search("编程", 2, 2).expect("search page 2");
    assert_eq!(page2.results.len(), page1.total - 2);

    // Test no-match query
    let no_match = engine.search("zzz_nonexistent_zzz", 1, 10).expect("no match");
    assert_eq!(no_match.total, 0);

    // Cleanup
    drop(engine);
    fs::remove_dir_all(&index_dir).ok();
}
