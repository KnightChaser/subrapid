// src/subdomains.rs

use std::collections::{HashMap, HashSet};

use colored::Colorize;
use url::Url;

/// Holds subdomains and their paths
#[derive(Debug, Default, Clone)]
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
    pub fn add_url(&mut self, url: &Url, root_domain: &str) -> bool {
        // host must exist
        let host = match url.host_str() {
            Some(h) => h.to_lowercase(),
            None => return false,
        };

        // Check if host belongs to root_domain
        // Either exact match, or ends with ".root_domain"
        let suffix = format!(".{}", root_domain);
        if host != root_domain && !host.ends_with(&suffix) {
            return false;
        }

        // Normalize: remove query + fragment
        let mut normalized_url = url.clone();
        normalized_url.set_query(None);
        normalized_url.set_fragment(None);

        let host = normalized_url.host_str().unwrap().to_string();
        let path = normalized_url.path().to_string();

        // Insert into map
        let entry = self.inner.entry(host).or_insert_with(HashSet::new);
        let is_new_host = entry.is_empty(); // If it was empty, this is the first path.
        entry.insert(path);

        is_new_host
    }

    /// Pretty-print everything in the map
    #[allow(dead_code)]
    pub fn print(&self) {
        for (host, paths) in &self.inner {
            println!("{host}");
            for path in paths {
                println!("  {path}");
            }
        }
    }

    /// Print only subdomains (host part before the root domain),
    /// with the subdomain highlighted and root domain kept normal.
    ///
    /// Example:
    ///   host: "mail.stack.com", root_domain: "stack.com"
    ///   prints: "<cyan bold>mail</cyan bold>.stack.com"
    ///
    ///   host: "stack.com" (no subdomain) -> skipped.
    pub fn print_subdomains_only(&self, root_domain: &str) {
        let mut hosts: Vec<_> = self.inner.keys().collect();
        hosts.sort();

        for host in hosts {
            let host = host.as_str();

            let Some(stripped) = host.strip_suffix(root_domain) else {
                // should not happen, as we only store same-root-domain hosts
                continue;
            };

            let stripped = stripped.strip_suffix('.').unwrap_or(stripped);
            if stripped.is_empty() {
                // When host == root_domain, no subdomain part
                continue;
            }

            let sub = stripped.cyan().bold();
            println!("{sub}.{root_domain}");
        }
    }

    /// Merge another SubdomainMap into this one.
    pub fn merge_from(&mut self, other: SubdomainMap) {
        for (host, paths) in other.inner {
            self.inner
                .entry(host)
                .or_insert_with(HashSet::new)
                .extend(paths);
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
    if parts.len() <= 2 {
        return None;
    }

    Some(format!(
        "{}.{}",
        parts[parts.len() - 2],
        parts[parts.len() - 1]
    ))
}
