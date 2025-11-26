// src/source/dns_bruteforce.rs

use anyhow::Result;

use crate::sources::{DiscoveryConfig, SubdomainSource};
use crate::subdomains::SubdomainMap;

pub struct DnsBruteforce;

impl DnsBruteforce {
    pub fn new() -> Self {
        Self
    }
}

impl SubdomainSource for DnsBruteforce {
    fn name(&self) -> &'static str {
        "dns-bruteforce"
    }

    fn discover(&self, _cfg: &DiscoveryConfig) -> Result<SubdomainMap> {
        // TODO: laceholder implementation
        Ok(SubdomainMap::new())
    }
}
