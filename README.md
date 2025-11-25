
# subrapid

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

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
