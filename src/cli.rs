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
}
