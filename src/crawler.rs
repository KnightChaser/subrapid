// src/crawler.rs
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use url::Url;

use crate::fetch::fetch_body;
use crate::parse::extract_links;
use crate::subdomains::SubdomainMap;

/// Internal shared crawler state.
/// Protected by Arc<Mutex<...>> in the crawler.
struct CrawlerState {
    /// Frontier of URLs to crawl
    queue: VecDeque<Url>,

    /// Full set of URLs that we have already visited
    visited_urls: HashSet<String>,

    /// How many pages we have crawled per host
    host_page_count: HashMap<String, usize>,

    /// Limit per hsot (to avoid overloading)
    max_pages_per_host: usize,

    /// Discovered subdomains / URLs under the root domain
    sub_map: SubdomainMap,

    /// Number of currently active workers
    active: usize,
}

impl CrawlerState {
    fn new(start_url: Url, max_pages_per_host: usize) -> Self {
        let mut queue = VecDeque::new();
        let mut visited_urls = HashSet::new();
        let mut host_page_count = HashMap::new();

        if let Some(host) = start_url.host_str() {
            let host = host.to_lowercase();
            queue.push_back(start_url.clone());
            visited_urls.insert(start_url.to_string());
            host_page_count.insert(host, 1);
        } else {
            // No host? Still push the URL, but it won't go far probably.
            queue.push_back(start_url.clone());
            visited_urls.insert(start_url.as_str().to_string());
        }

        Self {
            queue,
            visited_urls,
            host_page_count,
            max_pages_per_host,
            sub_map: SubdomainMap::new(),
            active: 0,
        }
    }
}

enum WorkItem {
    Url(Url),
    Wait,
    Done,
}

/// Run a multi-threaded crawl and return the final subdomain map.
///
/// - `start_url`: initial URL to begin crawling (e.g. https://www.stackexchange.com)
/// - `root_domain`: root scope (e.g. "stackexchange.com")
/// - `worker_count`: number of worker threads
/// - `max_pages_per_host`: safety cap, e.g. 5 or 10
pub fn crawl(
    start_url: Url,
    root_domain: String,
    worker_count: usize,
    max_pages_per_host: usize,
) -> Result<SubdomainMap> {
    let state = Arc::new(Mutex::new(CrawlerState::new(start_url, max_pages_per_host)));

    let mut handles = Vec::new();

    for worker_id in 0..worker_count {
        let state = Arc::clone(&state);
        let root_domain = root_domain.clone();

        let handle = thread::spawn(move || {
            worker_loop(state, &root_domain, worker_id);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked >_<");
    }

    // Pull the sub_map out of the shared state
    let guard = state.lock().unwrap();
    let result = guard.sub_map.clone();
    Ok(result)
}

/// Start the crawling process from the given start_url.
fn worker_loop(state: Arc<Mutex<CrawlerState>>, root_domain: &str, worker_id: usize) {
    loop {
        let work = {
            let mut st = state.lock().unwrap();

            if let Some(url) = st.queue.pop_front() {
                // Take work and mark as active
                st.active += 1;
                WorkItem::Url(url)
            } else if st.active > 0 {
                // No work, but others are active: wait
                WorkItem::Wait
            } else {
                // No work and no active workers: done
                WorkItem::Done
            }
        };

        match work {
            WorkItem::Url(url) => {
                let res = process_url(&state, &url, root_domain, worker_id);
                if let Err(e) = res {
                    eprintln!("[!] [worker {}] Error processing {}: {}", worker_id, url, e);
                }

                let mut st = state.lock().unwrap();
                st.active -= 1;

                let current_pages = st.visited_urls.len();
                let max_possible = st.host_page_count.len() * st.max_pages_per_host;
                eprintln!(
                    "[~] [worker {worker_id} ({current_pages} queued, max {max_possible} possible)] Finished {}",
                    url
                );
            }
            WorkItem::Wait => {
                thread::sleep(Duration::from_millis(100));
            }
            WorkItem::Done => {
                break;
            }
        }
    }
}

/// Check if given host is in scope of the root_domain
/// Example:
///     root_domain: `example.com`
///     in scope: `example.com`, `sub.example.com`
///     out of scope: `other.com`, `example.org`, `sub.example.org`
fn host_in_scope(host: &str, root_domain: &str) -> bool {
    let host = host.to_lowercase();
    let root = root_domain.to_lowercase();

    if host == root {
        return true;
    }

    let suffix = format!(".{}", root);
    host.ends_with(&suffix)
}

fn process_url(
    state: &Arc<Mutex<CrawlerState>>,
    url: &Url,
    root_domain: &str,
    worker_id: usize,
) -> Result<()> {
    let body = fetch_body(url.as_str())?;
    let links = extract_links(&body, url)?;

    let mut st = state.lock().unwrap();

    for link in links {
        let host = match link.host_str() {
            Some(h) => h.to_lowercase(),
            None => continue,
        };

        // Only follow links inside the root domain scope
        if !host_in_scope(&host, root_domain) {
            continue;
        }

        // Always record in the subdomain map, even if we don't crawl the page
        // And check if this host is newly discovered
        let is_new_host = st.sub_map.add_url(&link, root_domain);

        // Announce new subdomain (host != root_domain)
        let root = root_domain.to_lowercase();
        if is_new_host && host != root {
            let current_pages = st.visited_urls.len();
            let max_possible = st.host_page_count.len() * st.max_pages_per_host;
            eprintln!(
                "[~] [worker {worker_id} ({current_pages} queued so far, max {max_possible})] \
                 Discovered subdomain `{}` at `{}`!",
                host, root_domain
            );
        }

        // Decide whether to crawl this URL or not
        let url_str = link.as_str().to_string();
        if st.visited_urls.contains(&url_str) {
            continue;
        }

        // Read the current count by value (ends the immutable borrow here)
        let current_count = st.host_page_count.get(&host).copied().unwrap_or(0);
        if current_count >= st.max_pages_per_host {
            continue;
        }

        let count = st.host_page_count.entry(host.clone()).or_insert(0);
        *count += 1;
        st.visited_urls.insert(url_str);
        st.queue.push_back(link);
    }

    Ok(())
}
