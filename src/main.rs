// src/main.rs

mod cli;
mod crawler;
mod fetch;
mod logging;
mod parse;
mod subdomains;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use url::Url;

use crate::cli::Cli;
use crate::crawler::{CrawlConfig, crawl};
use crate::subdomains::extract_root_domain;

fn main() -> Result<()> {
    let args = Cli::parse();

    // Parse starting URL
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

    // Decide root domain
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

    let config = CrawlConfig {
        start_url: start_url.clone(),
        root_domain: root_domain.clone(),
        workers: args.workers,
        max_pages_per_host: args.max_pages_per_host,
    };
    let sub_map = crawl(config)?;

    println!(
        "{}",
        format!("Discovered subdomains under '{}':", root_domain)
            .green()
            .bold()
    );
    sub_map.print_subdomains_only(&root_domain);

    Ok(())
}
