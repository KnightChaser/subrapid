// src/sources/mod.rs

pub mod crtsh;
pub mod html_crawler;
pub mod wayback;

use anyhow::Result;
use url::Url;

use crate::subdomains::SubdomainMap;

/// Shared config for all discovery strategies.
#[derive(Clone, Debug)]
pub struct DiscoveryConfig {
    /// The URL to start crawling from.
    pub start_url: Url,

    /// The root domain to limit discovery to.
    pub root_domain: String,

    /// Number of concurrent workers to use.
    pub workers: usize,

    /// Maximum number of pages to crawl per host.
    pub max_pages_per_host: usize,
}

/// A pluggable source of subdomains (HTML crawling, DNS bruteforce, CT logs, ...).
pub trait SubdomainSource: Send + Sync {
    /// Returns the name of this discovery source.
    fn name(&self) -> &'static str;

    /// Discovers subdomains according to the given config.
    fn discover(&self, cfg: &DiscoveryConfig) -> Result<SubdomainMap>;
}
