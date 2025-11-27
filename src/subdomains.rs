// src/subdomains.rs

use std::collections::{HashMap, HashSet};

use colored::Colorize;
use psl::domain_str;
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

/// Extract the registrable ("root") domain using the Public Suffix List.
///
/// Examples:
/// - "www.stackoverflow.com" -> Some("stackoverflow.com")
/// - "example.co.uk"         -> Some("example.co.uk")
/// - "co.uk"                 -> None (not registrable itself)
pub fn extract_root_domain(host: &str) -> Option<String> {
    domain_str(host).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_root_domain;

    #[test]
    fn test_extract_root_domain_basic() {
        assert_eq!(
            extract_root_domain("www.stackoverflow.com").as_deref(),
            Some("stackoverflow.com")
        );
        assert_eq!(
            extract_root_domain("stackoverflow.com").as_deref(),
            Some("stackoverflow.com")
        );
    }

    #[test]
    fn test_extract_root_domain_multi_tld() {
        assert_eq!(
            extract_root_domain("a.b.example.co.uk").as_deref(),
            Some("example.co.uk")
        );
        assert_eq!(
            extract_root_domain("example.co.uk").as_deref(),
            Some("example.co.uk")
        );
    }
}
