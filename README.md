# nsswitch: Name Service Switch for Rust

This crate will eventually contain a pure-Rust implementation of the
name service switch facilities provided by many Unix-style systems on all
platforms supported by Rust. It will allow looking up various types of
network-related names according to the system configuration.

For the moment, the crate is too young to be on crates.io.


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
nsswitch = "0.1.0"
```

Then, add this to your crate root:

```rust
extern crate nsswith
```

