// src/sources/wayback.rs

use std::collections::HashSet;
use std::time::Duration;

use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value;
use url::Url;

use crate::sources::{DiscoveryConfig, SubdomainSource};
use crate::subdomains::SubdomainMap;

pub struct WaybackArchive;

impl WaybackArchive {
    pub fn new() -> Self {
        Self
    }
}

impl SubdomainSource for WaybackArchive {
    fn name(&self) -> &'static str {
        "Wayback Machine"
    }

    fn discover(&self, cfg: &DiscoveryConfig) -> Result<SubdomainMap> {
        eprintln!("[*] Querying Wayback Machine API...");

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("subrapid-knightchaser/0.1")
            .build()
            .context("Failed to build HTTP client for Wayback Machine")?;

        let query_url = format!(
            "http://web.archive.org/cdx/search/cdx?url={}&output=json&limit=100000&fl=original",
            cfg.root_domain
        );

        let resp = client.get(&query_url).send().with_context(|| {
            format!(
                "Failed to send request to Wayback Machine for domain {}",
                cfg.root_domain
            )
        })?;

        if !resp.status().is_success() {
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(SubdomainMap::new());
            }
            anyhow::bail!(
                "Wayback Machine returned non-success status code: {}",
                resp.status()
            );
        }

        let text = resp
            .text()
            .context("Failed to read Wayback response body")?;
        let root: Value = serde_json::from_str(&text).context("Failed to parse Wayback JSON")?;
        let rows = root
            .as_array()
            .context("Wayback response was not a JSON array")?;

        let mut map = SubdomainMap::new();
        let mut processed_hosts = HashSet::new();

        // NOTE::
        // -1 because first row is header. Example output be like:
        // ```json
        // [["original"],
        // ["http://www.youtube.com:80/"],
        // ["http://www.youtube.com:80/"],
        // ["http://www.youtube.com:80/"],
        // ...
        // ```
        eprintln!("[~] Processing {} historical records...", rows.len() - 1);

        // NOTE:
        // Skip the header row
        for row in rows.iter().skip(1) {
            if let Some(url_str) = row.get(0).and_then(|v| v.as_str()) {
                // Parse URL
                let Ok(parsed_url) = Url::parse(url_str) else {
                    continue;
                };

                let Some(host_str) = parsed_url.host_str() else {
                    continue;
                };

                let host_clean = host_str.trim().to_lowercase();

                // 1. Deduplication Filter (HashSet)
                // If we've seen this host in this loop, skip it immediately.
                if !processed_hosts.insert(host_clean.clone()) {
                    continue;
                }

                // 2. Add to map
                if map.add_url(&parsed_url, &cfg.root_domain) {
                    eprintln!(
                        "{} Discovered potential (sub)domain {} via Wayback Machine",
                        "[+]".green().bold(),
                        format!("{}", host_clean).bold()
                    );
                }
            }
        }

        Ok(map)
    }
}
