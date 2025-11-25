// src/cli.rs

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "subrapid",
    version = "0.1",
    author = "knightchaser",
    about = "A tool to gather subdomains from a given URL"
)]
pub struct Cli {
    /// The starting URL (e.g. "https://example.com")
    pub url: String,

    /// Root domain to scope to (e.g. stackexchange.com).
    /// If omitted, you can derive it from the URL's host in main().
    #[arg(long)]
    pub root_domain: Option<String>,

    /// Number of worker threads
    #[arg(long, default_value_t = 8)]
    pub workers: usize,

    /// Maximum pages to crawl per host (to avoid explosion)
    #[arg(long, default_value_t = 5)]
    pub max_pages_per_host: usize,
}
