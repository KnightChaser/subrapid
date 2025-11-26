// src/sources/crtsh.rs

use crate::sources::{DiscoveryConfig, SubdomainSource};
use crate::subdomains::SubdomainMap;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;
use url::Url;

pub struct CrtSh;

impl CrtSh {
    pub fn new() -> Self {
        CrtSh
    }
}

/// Represents a single item from the crt.sh JSON response.
/// e.g. {"name_value":"example.com"}
#[derive(Deserialize, Debug)]
struct CrtShEntry {
    name_value: String,
}

impl SubdomainSource for CrtSh {
    fn name(&self) -> &'static str {
        "crt.sh"
    }

    fn discover(&self, cfg: &DiscoveryConfig) -> Result<SubdomainMap> {
        // crt.sh query syntax: %.example.com returns all subdomains
        let query_url = format!("https://crt.sh/?q={}&output=json", cfg.root_domain);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("subrapid-knightchaser/0.1")
            .build()
            .context("Failed to build HTTP client for crt.sh")?;

        let resp = client.get(&query_url).send().with_context(|| {
            format!(
                "Failed to send request to crt.sh for domain {}",
                cfg.root_domain
            )
        })?;

        if !resp.status().is_success() {
            anyhow::bail!("crt.sh returned non-success status code: {}", resp.status());
        }

        // Parse the JSON list from crt.sh
        let entries: Vec<CrtShEntry> = resp
            .json()
            .context("Failed to parse JSON response from crt.sh")?;
        let mut map = SubdomainMap::new();

        for entry in entries {
            for raw_domain in entry.name_value.split('\n') {
                let domain = raw_domain.trim().to_lowercase();

                // Skip wildcards
                if domain.contains('*') {
                    continue;
                }

                // SubdomainMa expects a Url, so we prepend "https://".
                if let Ok(fake_url) = Url::parse(&format!("https://{}", domain)) {
                    map.add_url(&fake_url, &cfg.root_domain);
                }
            }
        }

        Ok(map)
    }
}
