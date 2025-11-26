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

pub fn extract_csp_links(csp_header: &str) -> Vec<Url> {
    let mut out = Vec::new();

    // NOTE:
    // CSP format: "directive value1 value2; directive2 value3 ..."
    for directive in csp_header.split(';') {
        let parts: Vec<&str> = directive.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        // The first part is the directive name, (e.g., "default-src", "script-src", etc.)
        // So skip it and process the rest as URLs or sources.
        for &token in &parts[1..] {
            let cleaned_token = token.replace('\'', "").replace('"', "");
            if cleaned_token == "self"
                || cleaned_token == "none"
                || cleaned_token.starts_with("nonce-")
                || cleaned_token == "trusted-types-eval"
                || cleaned_token == "unsafe-eval"
                || cleaned_token == "wasm-unsafe-eval"
                || cleaned_token == "unsafe-inline"
                || cleaned_token == "unsafe-hashes"
                || cleaned_token == "inline-speculation-rules"
                || cleaned_token == "strict-dynamic"
                || cleaned_token == "report-sample"
                || cleaned_token == "data:"
                || cleaned_token == "blob:"
                || cleaned_token == "filesystem:"
            {
                continue;
            }

            // Attempt to parse as URL
            if let Ok(url) = Url::parse(&cleaned_token) {
                out.push(url);
                continue;
            }

            // If it's just host?
            if let Ok(url) = Url::parse(&format!("https://{}", cleaned_token)) {
                out.push(url);
            }
        }
    }

    out
}
