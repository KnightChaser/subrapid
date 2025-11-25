mod cli;
mod crawler;
mod fetch;
mod parse;
mod subdomains;

use anyhow::{Context, Result};
use clap::Parser;
use url::Url;

use crate::cli::Cli;
use crate::subdomains::extract_root_domain;

fn main() -> Result<()> {
    let args = Cli::parse();

    // Parse starting URL
    let start_url =
        Url::parse(&args.url).with_context(|| format!("invalid start URL: {}", args.url))?;

    let host = start_url
        .host_str()
        .context("start URL has no host")?
        .to_string();

    // Compute naive root-domain, e.g. "stackoverflow.com"
    let root_domain = extract_root_domain(&host)
        .with_context(|| format!("could not extract root domain from host: {}", host))?;

    eprintln!("[*] Start URL: {}", start_url);
    eprintln!("[*] Host: {}", host);
    eprintln!("[*] Root domain: {}", root_domain);

    let worker_count = 8;
    eprintln!("[*] Starting crawl with {} workers...", worker_count);

    let sub_map =
        crawler::crawl(start_url, root_domain.clone(), worker_count).context("crawl failed")?;

    if args.subdomains_only {
        println!("Subdomains under root domain '{}':", root_domain);
        sub_map.print_subdomains_only(&root_domain);
    } else {
        println!("Subdomains and paths under root domain '{}':", root_domain);
        sub_map.print();
    }

    Ok(())
}
