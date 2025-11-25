// src/subdomains.rs
use std::collections::{HashMap, HashSet};

use url::Url;

/// Holds subdomains and their paths
#[derive(Debug, Default)]
pub struct SubdomainMap {
    // host -> set of paths
    inner: HashMap<String, HashSet<String>>,
}

impl SubdomainMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Add a URL if it belongs to the given root domain.
    pub fn add_url(&mut self, url: &Url, root_domain: &str) {
        // host must exist
        let host = match url.host_str() {
            Some(h) => h.to_lowercase(),
            None => return,
        };

        // Only accept the same-root-domain hosts,
        // e.g. *.example.com for root_domain example.com
        if !host.ends_with(root_domain) {
            return;
        }

        // Normalize: remove query + fragment
        let mut normalized_url = url.clone();
        normalized_url.set_query(None);
        normalized_url.set_fragment(None);

        let host = normalized_url.host_str().unwrap().to_string();
        let path = normalized_url.path().to_string();

        // Insert into map
        let entry = self.inner.entry(host).or_insert_with(HashSet::new);
        entry.insert(path);
    }

    /// Pretty-print everything in the map
    pub fn print(&self) {
        for (host, paths) in &self.inner {
            println!("{host}");
            for path in paths {
                println!("  {path}");
            }
        }
    }
}

/// Very naive "root domain" extractor.
/// "www.stackoverflow.com" -> "stackoverflow.com"
/// "chat.stackoverflow.com" -> "stackoverflow.com"
///
/// WARN: This is *not* correct for all TLDs (e.g. .co.uk),
/// but fine as a first step.
///
/// TODO: Use a proper public suffix list library.
pub fn extract_root_domain(host: &str) -> Option<String> {
    let parts: Vec<_> = host.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let root = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    Some(root)
}
