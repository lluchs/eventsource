//! # EventSource
//!
//! EventSource is a Rust library for reading from Server-Sent Events endpoints. It transparently
//! sends HTTP requests and only exposes a stream of events to the user. It handles automatic
//! reconnection and parsing of the `text/event-stream` data format.
//!
//! # Examples
//!
//! ```no_run
//! # extern crate hyper;
//! # extern crate eventsource;
//! let url = hyper::Url::parse("http://example.com/").unwrap();
//! let client = eventsource::Client::new(url);
//! for event in client {
//!     println!("{}", event.unwrap());
//! }
//! ```
//!

#[macro_use] extern crate hyper;

mod error;

pub use error::Error;

use std::fmt;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};
use hyper::client::{Client as HyperClient};
use hyper::client::response::Response;
use hyper::header::{self, Headers};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::Url;

const DEFAULT_RETRY: u64 = 5000;

header! { (LastEventID, "Last-Event-ID") => [String] }

/// A client for a Server-Sent Events endpoint.
///
/// Read events by iterating over the client.
pub struct Client {
    hc: HyperClient,
    reader: Option<BufReader<Response>>,
    url: Url,
    last_event_id: Option<String>,
    last_try: Option<Instant>,

    /// Reconnection time in milliseconds. Note that the reconnection time can be changed by the
    /// event stream, so changing this may not make a difference.
    pub retry: Duration,
}

/// A single Server-Sent Event.
#[derive(Debug)]
pub struct Event {
    /// Corresponds to the `id` field.
    pub id: Option<String>,
    /// Corresponds to the `event` field.
    pub event_type: Option<String>,
    /// All `data` fields concatenated by newlines.
    pub data: String,
}

enum ParseResult {
    Next,
    Dispatch,
}

impl Client {
    /// Constructs a new EventSource client for the given URL.
    ///
    /// This does not start an HTTP request.
    pub fn new(url: Url) -> Client {
        Client {
            hc: HyperClient::new(),
            reader: None,
            url: url,
            last_event_id: None,
            retry: Duration::from_millis(DEFAULT_RETRY),
            last_try: None,
        }
    }

    fn next_request(&self) -> hyper::error::Result<Response> {
        let mut headers = Headers::new();
        if let Some(ref id) = self.last_event_id {
            headers.set(LastEventID(id.clone()));
        }
        self.hc.get(self.url.clone()).headers(headers).send()
    }

    fn parse_event_line(&mut self, line: &str, event: &mut Event) -> ParseResult {
        let line = if line.ends_with('\n') { &line[0..line.len()-1] } else { line };
        if line == "" {
            ParseResult::Dispatch
        } else {
            let (field, value) = if let Some(pos) = line.find(':') {
                let (f, v) = line.split_at(pos);
                // Strip : and an optional space.
                let v = &v[1..];
                let v = if v.starts_with(' ') { &v[1..] } else { v };
                (f, v)
            } else {
                (line, "")
            };
            
            match field {
                "event" => { event.event_type = Some(value.to_string()); },
                "data" => { event.data.push_str(value); event.data.push('\n'); },
                "id" => { event.id = Some(value.to_string()); self.last_event_id = Some(value.to_string()); }
                "retry" => {
                    if let Ok(retry) = value.parse::<u64>() {
                        self.retry = Duration::from_millis(retry);
                    }
                },
                _ => () // ignored
            }

            ParseResult::Next
        }
    }
}

// Helper macro for Option<Result<...>>
macro_rules! try_option {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => return Some(Err(::std::convert::From::from(err))),
    });
}

/// Iterate over the client to get events.
///
/// HTTP requests are made transparently while iterating.
impl Iterator for Client {
    type Item = Result<Event, Error>;

    fn next(&mut self) -> Option<Result<Event, Error>> {
        if self.reader.is_none() {
            // We may have to wait for the next request.
            if let Some(last_try) = self.last_try {
                let elapsed = last_try.elapsed();
                if elapsed < self.retry {
                    std::thread::sleep(self.retry - elapsed);
                }
            }
            // Set here in case the request fails.
            self.last_try = Some(Instant::now());

            let req = try_option!(self.next_request());
            // We can only work with successful requests.
            if !req.status.is_success() {
                return Some(Err(Error::Http(req.status)));
            }
            // Verify Content-Type = text/event-stream.
            match req.headers.get() {
                Some(&header::ContentType(Mime(TopLevel::Text, SubLevel::EventStream, _))) => (), // ok
                ct => return Some(Err(Error::InvalidContentType(ct.map(|x| x.clone())))),
            }
            let r = BufReader::new(req);
            self.reader = Some(r);
        }
        let mut event = Event::new();
        let mut line = String::new();

        // We can't have a mutable reference to the reader because of the &mut self call below.
        // The first unwrap() is safe as we're checking that above.
        while try_option!(self.reader.as_mut().unwrap().read_line(&mut line)) > 0 {
            match self.parse_event_line(&line, &mut event) {
                ParseResult::Dispatch => return Some(Ok(event)),
                ParseResult::Next => (),
            }
            line.clear();
        }
        // EOF, retry after timeout
        self.last_try = Some(Instant::now());
        self.reader = None;
        self.next()
    }
}

impl Event {
    fn new() -> Event {
        Event {
            id: None,
            event_type: None,
            data: "".to_string(),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref id) = self.id {
            try!(write!(f, "id: {}\n", id));
        }
        if let Some(ref event_type) = self.event_type {
            try!(write!(f, "event: {}\n", event_type));
        }
        for line in self.data.lines() {
            try!(write!(f, "data: {}\n", line));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_event_display() {
        assert_eq!(
            "data: hello world\n",
            Event { id: None, event_type: None, data: "hello world".to_string() }.to_string());
        assert_eq!(
            "id: foo\ndata: hello world\n",
            Event { id: Some("foo".to_string()), event_type: None, data: "hello world".to_string() }.to_string());
        assert_eq!(
            "event: bar\ndata: hello world\n",
            Event { id: None, event_type: Some("bar".to_string()), data: "hello world".to_string() }.to_string());
    }

    #[test]
    fn multiline_event_display() {
        assert_eq!(
            "data: hello\ndata: world\n",
            Event { id: None, event_type: None, data: "hello\nworld".to_string() }.to_string());
        assert_eq!(
            "data: hello\ndata: \ndata: world\n",
            Event { id: None, event_type: None, data: "hello\n\nworld".to_string() }.to_string());
    }
}
