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

/// A naive "root domain" extractor.
/// "www.stackoverflow.com" -> "stackoverflow.com"
/// "chat.stackoverflow.com" -> "stackoverflow.com"
///
/// WARN: This is *not* a complete implementation for all valid TLDs (e.g. .co.uk),
/// but fine as a first step.
pub fn extract_root_domain(host: &str) -> Option<String> {
    // NOTE:
    // Example TLDs that consist of two parts:
    const SPECIAL_TLDS: &[&str] = &["co.uk", "org.uk", "gov.uk", "co.kr", "ac.kr"];

    let host = host.to_lowercase();

    // 1. Check special multi-label TLDs
    for tld in SPECIAL_TLDS {
        if host.ends_with(tld) {
            let without_tld = host.strip_suffix(tld)?.strip_suffix('.')?;

            // take last label before TLD
            if let Some(last_dot) = without_tld.rfind('.') {
                let sld = &without_tld[last_dot + 1..];
                return Some(format!("{}.{}", sld, tld));
            } else {
                // host is just like "example.co.uk"
                return Some(format!("{}.{}", without_tld, tld));
            }
        }
    }

    // 2. Fallback: naive last-two-labels approach
    let parts: Vec<_> = host.split('.').collect();
    if parts.len() >= 2 {
        return None;
    }

    Some(format!(
        "{}.{}",
        parts[parts.len() - 2],
        parts[parts.len() - 1]
    ))
}
