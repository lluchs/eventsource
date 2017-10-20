//! # EventSource
//!
//! EventSource is a Rust library for reading from Server-Sent Events endpoints. It transparently
//! sends HTTP requests and only exposes a stream of events to the user. It handles automatic
//! reconnection and parsing of the `text/event-stream` data format.
//!
//! # Examples
//!
//! ```no_run
//! extern crate eventsource;
//! extern crate reqwest;
//! use eventsource::reqwest::Client;
//! use reqwest::Url;
//!
//! fn main() {
//!     let client = Client::new(Url::parse("http://example.com").unwrap());
//!     for event in client {
//!         println!("{}", event.unwrap());
//!     }
//! }
//! ```
//!

#[macro_use]
extern crate error_chain;

// Generic text/event-stream parsing and serialization.
pub mod event;

// HTTP interface
#[cfg(feature = "with-reqwest")]
pub mod reqwest;
