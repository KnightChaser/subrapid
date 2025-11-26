// src/logging.rs

use colored::Colorize;
use url::Url;

/// Snapshot of crawl progress used for logging.
#[derive(Clone, Copy, Debug)]
pub struct CrawlerStats {
    pub visited_pages: usize,
    pub hosts_seen: usize,
    pub max_pages_per_host: usize,
}

impl CrawlerStats {
    /// Calculate how many pages could have been visited based on hosts seen.
    pub fn max_possible_pages(&self) -> usize {
        self.hosts_seen * self.max_pages_per_host
    }
}

pub fn log_worker_error(worker_id: usize, url: &Url, err: &anyhow::Error, stats: &CrawlerStats) {
    eprintln!(
        "{} {} Error processing {}: {}",
        "[!]".red().bold(),
        format!(
            "[worker {} ({} visited, max {} possible)]",
            worker_id,
            stats.visited_pages,
            stats.max_possible_pages()
        )
        .yellow(),
        url,
        err
    );
}

pub fn log_worker_finished(worker_id: usize, url: &Url, stats: &CrawlerStats) {
    eprintln!(
        "{} {} Finished {}",
        "[~]".blue().bold(),
        format!(
            "[worker {} ({} visited, max {} possible)]",
            worker_id,
            stats.visited_pages,
            stats.max_possible_pages()
        )
        .cyan(),
        url
    );
}

pub fn log_new_subdomain(worker_id: usize, host: &str, root_domain: &str, stats: &CrawlerStats) {
    eprintln!(
        "{} {} Discovered subdomain `{}` at `{}`!",
        "[+]".green().bold(),
        format!(
            "[worker {} ({} visited, max {} possible)]",
            worker_id,
            stats.visited_pages,
            stats.max_possible_pages()
        )
        .cyan(),
        host.bold(),
        root_domain
    );
}
