mod cli;
mod fetch;
mod parse;

use anyhow::Result;
use clap::Parser;
use url::Url;

use crate::cli::Cli;
use crate::fetch::fetch_body;
use crate::parse::extract_links;

fn main() -> Result<()> {
    let args = Cli::parse();

    let body = fetch_body(&args.url)?;

    if args.links_only {
        let base = Url::parse(&args.url)?;
        let links = extract_links(&body, &base)?;

        println!("Found {} links:", links.len());
        for link in links {
            println!("{}", link);
        }
    } else {
        println!("{}", body);
    }

    Ok(())
}
