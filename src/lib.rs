//! # EventSource
//!
//! EventSource is a Rust library for reading from Server-Sent Events endpoints. It transparently
//! sends HTTP requests and only exposes a stream of events to the user. It handles automatic
//! reconnection and parsing of the `text/event-stream` data format.
//!
//! # Examples
//!
//! ```no_run
//! use eventsource::curl::Client;
//! let client = Client::new("http://example.com");
//! for event in client {
//!     println!("{}", event.unwrap());
//! }
//! ```
//!

#[macro_use]
extern crate error_chain;

// Generic text/event-stream parsing and serialization.
pub mod event;

// HTTP interface
#[cfg(feature = "with-curl")]
pub mod curl;
