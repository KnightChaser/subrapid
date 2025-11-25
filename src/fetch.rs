// src/fetch.rs

use anyhow::{Context, Result};

pub fn fetch_body(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
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

    let body = resp
        .text()
        .context("failed to read response body as text")?;

    Ok(body)
}
