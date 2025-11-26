// src/main.rs

mod cli;
mod fetch;
mod logging;
mod parse;
mod sources;
mod subdomains;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use url::Url;

use crate::cli::Cli;
use crate::sources::dns_bruteforce::DnsBruteforce;
use crate::sources::html_crawler::HtmlCrawler;
use crate::sources::{DiscoveryConfig, SubdomainSource};
use crate::subdomains::{SubdomainMap, extract_root_domain};

fn main() -> Result<()> {
    let args = Cli::parse();

    let start_url =
        Url::parse(&args.url).with_context(|| format!("invalid start URL: {}", args.url))?;

    let host = start_url
        .host_str()
        .with_context(|| {
            format!(
                "Cannot derive root domain from URL {} without host",
                args.url
            )
        })?
        .to_lowercase();

    let root_domain = if let Some(rd) = args.root_domain {
        rd.to_lowercase()
    } else if let Some(root) = extract_root_domain(&host) {
        root
    } else {
        return Err(anyhow::anyhow!(
            "Cannot derive root domain from host: {}. Please specify --root-domain",
            host
        ));
    };

    let cfg = DiscoveryConfig {
        start_url: start_url.clone(),
        root_domain: root_domain.clone(),
        workers: args.workers,
        max_pages_per_host: args.max_pages_per_host,
    };

    let sources: Vec<Box<dyn SubdomainSource>> =
        vec![Box::new(HtmlCrawler::new()), Box::new(DnsBruteforce::new())];

    let mut combined = SubdomainMap::new();
    for src in sources {
        eprintln!(
            "{}",
            format!("[*] Running source: {}", src.name())
                .magenta()
                .bold()
        );
        let map = src.discover(&cfg)?;
        combined.merge_from(map);
    }

    println!(
        "{}",
        format!("Discovered subdomains under '{}':", root_domain)
            .green()
            .bold()
    );
    combined.print_subdomains_only(&root_domain);

    Ok(())
}
