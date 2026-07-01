/// Generate a snippet from `body` showing context around `query_terms`.
/// Returns a string with "..." for gaps between non-overlapping match windows.
pub fn generate_snippet(body: &str, query_terms: &[String], window: usize) -> String {
    let body_lower = body.to_lowercase();
    let terms_lower: Vec<String> = query_terms.iter().map(|t| t.to_lowercase()).collect();

    // Find all match spans in the body
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for term in &terms_lower {
        for (start, _) in body_lower.match_indices(term.as_str()) {
            spans.push((start, start + term.len()));
        }
    }

    if spans.is_empty() {
        // No match found — return the first `window * 2` characters
        let end = body.char_indices()
            .take(window * 2)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(body.len());
        return format!("{}...", &body[..end.min(body.len())]);
    }

    // Sort and merge overlapping spans
    spans.sort_by_key(|s| s.0);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for span in spans {
        if let Some(last) = merged.last_mut() {
            if span.0 <= last.1 {
                last.1 = last.1.max(span.1);
            } else {
                merged.push(span);
            }
        } else {
            merged.push(span);
        }
    }

    // Expand each merged span to include context window
    let mut windows: Vec<(usize, usize)> = Vec::new();
    for (start, end) in &merged {
        let ctx_start = start.saturating_sub(window);
        let ctx_end = (end + window).min(body.len());
        windows.push((ctx_start, ctx_end));
    }

    // Merge overlapping windows
    let mut final_windows: Vec<(usize, usize)> = Vec::new();
    for w in windows {
        if let Some(last) = final_windows.last_mut() {
            if w.0 <= last.1 + 20 {
                // Allow small gaps to be merged (20 char tolerance)
                last.1 = last.1.max(w.1);
            } else {
                final_windows.push(w);
            }
        } else {
            final_windows.push(w);
        }
    }

    // Build snippet with "..." separators
    let mut result = String::new();
    for (i, (start, end)) in final_windows.iter().enumerate() {
        if i > 0 {
            result.push_str(" ... ");
        }
        // Adjust to nearest char boundary
        let s = find_char_boundary(body, *start);
        let e = find_char_boundary(body, *end);
        let frag = &body[s..e].trim();
        result.push_str(frag);
    }

    result
}

fn find_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    if s.is_char_boundary(pos) {
        pos
    } else {
        (0..=pos).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_finds_keyword() {
        let body = "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.";
        let snippet = generate_snippet(body, &["Rust".to_string()], 20);
        assert!(snippet.contains("Rust"));
    }

    #[test]
    fn test_snippet_no_match() {
        let body = "This is a test paragraph.";
        let snippet = generate_snippet(body, &["nonexistent".to_string()], 20);
        assert!(!snippet.is_empty());
    }

    #[test]
    fn test_snippet_chinese() {
        let body = "Rust 是一门系统编程语言，具有内存安全和高性能的特点。很多人喜欢用它来构建网络服务。";
        let snippet = generate_snippet(body, &["Rust".to_string()], 10);
        assert!(snippet.contains("Rust"));
    }
}
