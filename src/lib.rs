//! Querying the System’s Network Name Database
//!
//! This crate allows querying the database of network-related names
//! similar to the what can be found in POSIX’s `netdb.h`. The queries
//! are performed in a way appropriate for the platform and consider the
//! system’s configuration.
//!
//! > **Note:** This is not yet true for this initial release. It performs
//! > queries in a hard-wired fashion and probably only works correctly
//! > on Unix-y systems.
//!
//! For each query, there are synchronous functions (generally prefixed with
//! `get_`) as well as asynchronous functions return futures atop a Tokio
//! reactor core (starting with `poll_` for want of a better prefix).
//!
//! For each database, there is a submodule. Have a look at these modules
//! for more information.
//! 
extern crate domain;
extern crate futures;
extern crate tokio_core;

pub mod hosts;
