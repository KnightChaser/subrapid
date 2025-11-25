// src/crawler.rs
use std::collections::{HashSet, VecDeque};
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
    queue: VecDeque<Url>,
    visited_hosts: HashSet<String>, // Hosts that have been visited
    sub_map: SubdomainMap,
    /// Number of currently active workers
    active: usize,
}

impl CrawlerState {
    fn new(start_url: Url) -> Self {
        let mut queue = VecDeque::new();
        let mut visited_hosts = HashSet::new();

        if let Some(host) = start_url.host_str() {
            visited_hosts.insert(host.to_string());
        }

        queue.push_back(start_url);

        Self {
            queue,
            visited_hosts,
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
pub fn crawl(start_url: Url, root_domain: String, worker_count: usize) -> Result<SubdomainMap> {
    let state = Arc::new(Mutex::new(CrawlerState::new(start_url)));

    let mut handles = Vec::new();

    for _ in 0..worker_count {
        let state = Arc::clone(&state);
        let root_domain = root_domain.clone();

        let handle = thread::spawn(move || worker_loop(state, &root_domain));
        handles.push(handle)
    }

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    // Pull the sub_map out of the shared state
    let guard = state.lock().unwrap();
    let result = guard.sub_map.clone();
    Ok(result)
}

fn worker_loop(state: Arc<Mutex<CrawlerState>>, root_domain: &str) {
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
                let res = process_url(&state, &url, root_domain);
                if let Err(e) = res {
                    eprintln!("[!] [worker] Error processing {}: {}", url, e);
                }
                eprintln!("[~] [worker] Finished {}", url);

                let mut st = state.lock().unwrap();
                st.active -= 1;
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

fn process_url(state: &Arc<Mutex<CrawlerState>>, url: &Url, root_domain: &str) -> Result<()> {
    let body = fetch_body(url.as_str())?;
    let links = extract_links(&body, url)?;

    let mut st = state.lock().unwrap();

    for link in links {
        // Ensure the link has a host and is under the same root domain
        let host = match link.host_str() {
            Some(h) => h.to_lowercase(),
            None => continue,
        };

        if !host.ends_with(root_domain) {
            continue;
        }

        // Record in subdomain map
        st.sub_map.add_url(&link, root_domain);

        // Only crawl a host once;
        // if we've never visited this host, enqueue one URL for it
        if !st.visited_hosts.contains(&host) {
            st.visited_hosts.insert(host.to_lowercase());
            st.queue.push_back(link);
        } else {
            // Already visited this host; ignore the link
            continue;
        }
    }

    Ok(())
}
