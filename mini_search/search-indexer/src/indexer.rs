use std::path::Path;

use tantivy::{
    schema::Schema, Index, IndexReader, IndexWriter, TantivyDocument,
};
use tracing;

use crate::schema::build_schema;
use crate::tokenizer::JiebaTokenizer;

pub struct Indexer {
    schema: Schema,
    index: Index,
    writer: IndexWriter,
}

impl Indexer {
    /// Open an existing index at `index_path`, or create a new one if it doesn't exist.
    pub fn open_or_create(index_path: &Path) -> Result<Self, tantivy::TantivyError> {
        let schema = build_schema();

        let index = if index_path.exists() {
            Index::open_in_dir(index_path)?
        } else {
            std::fs::create_dir_all(index_path).map_err(|e| {
                tantivy::TantivyError::SystemError(format!(
                    "Failed to create index directory: {e}"
                ))
            })?;
            Index::create_in_dir(index_path, schema.clone())?
        };

        // Register jieba tokenizer
        index.tokenizers().register("jieba", JiebaTokenizer::new());

        let writer: IndexWriter<tantivy::TantivyDocument> =
            index.writer(50_000_000)?; // 50 MB memory budget

        tracing::info!("Indexer opened at {:?}", index_path);

        Ok(Self {
            schema,
            index,
            writer,
        })
    }

    /// Add a single document to the index (not yet committed).
    pub fn add_document(
        &mut self,
        url: &str,
        title: &str,
        body: &str,
        crawled_at: u64,
    ) -> Result<(), tantivy::TantivyError> {
        let url_field = self
            .schema
            .get_field("url")
            .expect("url field missing from schema");
        let title_field = self
            .schema
            .get_field("title")
            .expect("title field missing from schema");
        let body_field = self
            .schema
            .get_field("body")
            .expect("body field missing from schema");
        let crawled_at_field = self
            .schema
            .get_field("crawled_at")
            .expect("crawled_at field missing from schema");

        let mut doc = TantivyDocument::new();
        doc.add_text(url_field, url);
        doc.add_text(title_field, title);
        doc.add_text(body_field, body);
        doc.add_u64(crawled_at_field, crawled_at);

        self.writer.add_document(doc)?;
        Ok(())
    }

    /// Commit pending documents to disk. Returns the total number of committed docs.
    pub fn commit(&mut self) -> Result<u64, tantivy::TantivyError> {
        let opstamp = self.writer.commit()?;
        tracing::info!("Index committed, opstamp: {opstamp}");
        Ok(opstamp)
    }

    /// Get a reader for searching the index.
    pub fn reader(&self) -> Result<IndexReader, tantivy::TantivyError> {
        self.index.reader()
    }

    /// Return the tantivy Schema, so callers can get field handles.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Return a reference to the underlying tantivy Index.
    pub fn index(&self) -> &Index {
        &self.index
    }
}

impl Drop for Indexer {
    fn drop(&mut self) {
        // Best-effort flush on drop
        if let Err(e) = self.writer.commit() {
            tracing::error!("Failed to commit on drop: {e}");
        }
    }
}
