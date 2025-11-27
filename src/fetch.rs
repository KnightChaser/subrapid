// src/fetch.rs

use anyhow::{Context, Result};
use std::time::Duration;

pub struct FetchedPage {
    pub body: String,
    pub csp: Option<String>, // Content-Security-Policy header if present
}

/// Fetches the body and Content-Security-Policy header of the given URL.
pub fn fetch_page(url: &str) -> Result<FetchedPage> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("subrapid-knightchaser/0.1")
        .build()
        .context("failed to build HTTP client")?;

    let resp = client
        .get(url)
        .send()
        .with_context(|| format!("failed to GET {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("request failed with status: {}", resp.status());
    }

    let csp = resp
        .headers()
        .get("Content-Security-Policy")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body = resp
        .text()
        .context("failed to read response body as text")?;

    Ok(FetchedPage { body, csp })
}
