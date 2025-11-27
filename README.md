
# subrapid

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

**subrapid** is a fast and efficient subdomain discovery tool written in Rust. I made this one to learn Rust. It gathers subdomains by crawling a target URL and querying various sources like crt.sh and the Wayback Machine.

## Features

- **Multi-source Discovery**:
  - **HTML Crawler**: Crawls web pages starting from a given URL to find links.
  - **crt.sh**: Queries Certificate Transparency logs.
  - **Wayback Machine**: Checks the Internet Archive for historical subdomains.
- **Smart Scope**: Automatically derives the root domain or allows manual specification.
- **Performance**: Configurable worker threads for concurrent processing.
- **Safety**: Limits the number of pages crawled per host to prevent infinite loops or excessive traffic.

## Installation

This project includes `openssl` crate as a dependency with `vendored` feature enabled to build OpenSSL from source.
To install the required build tools, run the following commands based on your operating system:

Refer to the [`openssl` crate documentation](https://docs.rs/openssl/latest/openssl/#automatic), install the necessary build tools for your platform depending on your operating system.

### Note

On Fedora operating system, run the following as the documentation says.

```bash
sudo dnf install pkgconf perl-FindBin perl-IPC-Cmd openssl-devel
```

However, if you encounter issues during the build process, you may need to install additional packages:

```bash
sudo dnf install perl-File-Compare perl
```

### Building from Source

Clone the repository and build using Cargo:

```bash
git clone https://github.com/KnightChaser/subrapid.git
cd subrapid
cargo build --release
```

The binary will be available at `target/release/subrapid`.

## Usage

Run the tool by providing a starting URL:

```bash
./target/release/subrapid https://example.com
```

### Options

- `--root-domain <DOMAIN>`: Manually specify the root domain to scope the search (e.g., `example.com`). If omitted, it is derived from the URL.
- `--workers <NUM>`: Set the number of worker threads (default: 8).
- `--max-pages-per-host <NUM>`: Limit the number of pages to crawl per host (default: 5).

### Examples

**Basic scan:**
```bash
subrapid https://example.com
```

**Specify root domain explicitly:**
```bash
subrapid https://sub.example.com --root-domain example.com
```

**Increase concurrency and crawl depth:**
```bash
subrapid https://example.com --workers 20 --max-pages-per-host 10
```
