use scraper::{Html, Selector};
use url::Url;

pub struct ParsedPage {
    pub title: String,
    pub body: String,
    pub links: Vec<String>,
}

pub struct Parser;

impl Parser {
    pub fn parse(html: &str, base_url: &str) -> Result<ParsedPage, String> {
        let document = Html::parse_document(html);
        let base = Url::parse(base_url).map_err(|e| format!("Invalid base URL: {e}"))?;

        Ok(ParsedPage {
            title: extract_title(&document),
            body: extract_body(&document),
            links: extract_links(&document, &base),
        })
    }
}

fn extract_title(document: &Html) -> String {
    let selector = Selector::parse("title").unwrap();
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(""))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn extract_body(document: &Html) -> String {
    let selector = Selector::parse("body").unwrap();
    match document.select(&selector).next() {
        Some(body) => body
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        None => {
            // No <body> tag, get all text from root
            document
                .root_element()
                .text()
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

fn extract_links(document: &Html, base: &Url) -> Vec<String> {
    let selector = Selector::parse("a[href]").unwrap();
    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            let href = href.trim();
            if href.is_empty() || href.starts_with('#') {
                continue;
            }
            // Filter out non-http schemes
            if href.strip_prefix("javascript:").is_some() {
                continue;
            }
            if href.strip_prefix("mailto:").is_some() {
                continue;
            }
            if href.strip_prefix("tel:").is_some() {
                continue;
            }
            match base.join(href) {
                Ok(absolute) => {
                    let url_str = absolute.to_string();
                    // Only keep http(s) URLs
                    if url_str.starts_with("http://") || url_str.starts_with("https://") {
                        // Strip fragment
                        let clean = match absolute.fragment() {
                            Some(_) => {
                                let mut u = absolute.clone();
                                u.set_fragment(None);
                                u.to_string()
                            }
                            None => url_str,
                        };
                        links.push(clean);
                    }
                }
                Err(_) => continue,
            }
        }
    }

    links.sort();
    links.dedup();
    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Hello World</title></head><body></body></html>";
        let page = Parser::parse(html, "https://example.com").unwrap();
        assert_eq!(page.title, "Hello World");
    }

    #[test]
    fn test_extract_links_relative() {
        let html = r##"
            <html><body>
                <a href="/page1">Page 1</a>
                <a href="page2">Page 2</a>
                <a href="https://other.com/page3">Page 3</a>
                <a href="#section">Section</a>
            </body></html>
        "##;
        let page = Parser::parse(html, "https://example.com/dir/").unwrap();
        assert!(page.links.contains(&"https://example.com/page1".to_string()));
        assert!(page.links.contains(&"https://example.com/dir/page2".to_string()));
        assert!(page.links.contains(&"https://other.com/page3".to_string()));
        assert!(!page.links.contains(&"https://example.com/dir/#section".to_string()));
    }

    #[test]
    fn test_extract_body_text() {
        let html = r#"
            <html><body>
                <p>First paragraph.</p>
                <p>Second <strong>paragraph</strong>.</p>
            </body></html>
        "#;
        let page = Parser::parse(html, "https://example.com").unwrap();
        assert!(page.body.contains("First paragraph"));
        assert!(page.body.contains("paragraph"));
    }
}
