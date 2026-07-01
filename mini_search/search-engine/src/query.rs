use std::path::Path;

use serde::Serialize;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, Value},
    Index, IndexReader, TantivyDocument,
};

use search_indexer::{build_schema, JiebaTokenizer};

use crate::snippet::generate_snippet;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub results: Vec<SearchResult>,
}

pub struct QueryEngine {
    index: Index,
    title_field: Field,
    body_field: Field,
    url_field: Field,
}

impl QueryEngine {
    /// Open an existing index for searching.
    pub fn open(index_path: &Path) -> Result<Self, tantivy::TantivyError> {
        let schema = build_schema();
        let index = Index::open_in_dir(index_path)?;

        // Re-register the jieba tokenizer (needed for query parsing too)
        index.tokenizers().register("jieba", JiebaTokenizer::new());

        let title_field = schema.get_field("title").expect("title field");
        let body_field = schema.get_field("body").expect("body field");
        let url_field = schema.get_field("url").expect("url field");

        Ok(Self {
            index,
            title_field,
            body_field,
            url_field,
        })
    }

    /// Search the index for `query_str`. Returns paginated results with snippets.
    pub fn search(
        &self,
        query_str: &str,
        page: usize,
        page_size: usize,
    ) -> Result<SearchResponse, tantivy::TantivyError> {
        let reader: IndexReader = self.index.reader()?;
        let searcher = reader.searcher();

        // Build query parser with title and body fields; title has 2x boost
        let mut query_parser =
            QueryParser::for_index(&self.index, vec![self.title_field, self.body_field, self.url_field]);

        // Boost title matches 2x over body matches
        query_parser.set_field_boost(self.title_field, 2.0);

        let query = query_parser.parse_query(query_str)?;
        let offset = page.saturating_sub(1) * page_size;
        let limit = offset + page_size;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let total = top_docs.len();
        let paged: Vec<_> = top_docs.into_iter().skip(offset).take(page_size).collect();

        let query_terms: Vec<String> = extract_terms(query_str);

        let mut results = Vec::new();
        for (score, doc_address) in paged {
            let doc: TantivyDocument = searcher.doc(doc_address)?;

            let url = doc
                .get_first(self.url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let body = doc
                .get_first(self.body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let snippet = generate_snippet(&body, &query_terms, 80);

            results.push(SearchResult {
                url,
                title,
                snippet,
                score,
            });
        }

        Ok(SearchResponse {
            query: query_str.to_string(),
            total,
            page,
            page_size,
            results,
        })
    }

    /// Return the document count in the index.
    pub fn doc_count(&self) -> Result<u64, tantivy::TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        Ok(searcher.num_docs() as u64)
    }
}

/// Extract search terms from a raw query string (simple whitespace split).
fn extract_terms(query_str: &str) -> Vec<String> {
    query_str
        .split_whitespace()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
