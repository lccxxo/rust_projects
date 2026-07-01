use tantivy::schema::{
    IndexRecordOption, Schema, TextFieldIndexing, TextOptions, STORED,
};

/// Build the tantivy schema with jieba tokenizer configured on text fields.
pub fn build_schema() -> Schema {
    let mut builder = Schema::builder();

    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("jieba")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);

    let text_opts = TextOptions::default()
        .set_stored()
        .set_indexing_options(text_indexing);

    builder.add_text_field("url", text_opts.clone());
    builder.add_text_field("title", text_opts.clone());
    builder.add_text_field("body", text_opts);
    builder.add_u64_field("crawled_at", STORED | tantivy::schema::INDEXED);

    builder.build()
}
