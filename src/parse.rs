// src/parse.rs
use anyhow::Result;
use scraper::{Html, Selector};
use url::Url;

/// Extract all absolute URLs from anchor tags in the given HTML body,
pub fn extract_links(body: &str, base: &Url) -> Result<Vec<Url>> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("a").expect("Failed to parse selector");

    let mut out = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            // Skip empty / JS pseudo links quickly
            if href.starts_with('#') || href.starts_with("javascript:") {
                continue;
            }

            if let Ok(resolved) = base.join(href) {
                out.push(resolved);
            }
        }
    }

    Ok(out)
}
