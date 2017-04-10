# netdb: Network Database for Rust

This crate will eventually contain a pure-Rust implementation of queries
for network-related names similar to the functions found in POSIX’
`netdb.h`. The eventual implementation will be cross-platform and perform
the correct actions as configured for the particular system.

For the moment, the crate is too young to be on crates.io.


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
netdb = "0.1.0"
```

Then, add this to your crate root:

```rust
extern crate netdb
```


## Databases

The crate will contain the following databases:

- [ ] hosts
- [ ] networks
- [ ] protocols
- [ ] services

