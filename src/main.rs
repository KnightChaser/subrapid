mod cli;
mod fetch;
mod parse;
mod subdomains;

use anyhow::{Context, Result};
use clap::Parser;
use url::Url;

use crate::cli::Cli;
use crate::fetch::fetch_body;
use crate::parse::extract_links;
use crate::subdomains::{SubdomainMap, extract_root_domain};

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

    // Fetch body
    let body = fetch_body(start_url.as_str())?;

    // Extract links
    let links = extract_links(&body, &start_url)?;
    eprintln!("[*] Extracted {} links", links.len());

    let mut sub_map = SubdomainMap::new();

    for link in &links {
        sub_map.add_url(link, &root_domain);
    }

    if args.links_only {
        println!("All extracted links:");
        for link in links {
            println!("{}", link);
        }
    } else {
        println!("Subdomains and paths under root domain '{}':", root_domain);
        sub_map.print();
    }

    Ok(())
}
