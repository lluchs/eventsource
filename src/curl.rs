//! # CURL-based EventSource client

extern crate curl as libcurl;

mod errors {
    error_chain! {
        foreign_links {
            Curl(super::libcurl::Error);
        }
    }
}
use self::errors::*;

use std::str;
use std::time::{Duration, Instant};
use self::libcurl::easy::{Easy, List, WriteError};
use super::event::{Event, ParseResult, parse_event_line};

const DEFAULT_RETRY: u64 = 5000;

/// A client for a Server-Sent Events endpoint.
///
/// Read events by iterating over the client.
pub struct Client {
    handle: Easy,
    current_line: String,
    url: String,
    last_event_id: Option<String>,
    last_try: Option<Instant>,

    /// Reconnection time in milliseconds. Note that the reconnection time can be changed by the
    /// event stream, so changing this may not make a difference.
    pub retry: Duration,
}

impl Client {
    /// Constructs a new EventSource client for the given URL.
    ///
    /// This does not start an HTTP request.
    pub fn new(url: &str) -> Client {
        Client {
            handle: Easy::new(),
            current_line: "".into(),
            url: url.into(),
            last_event_id: None,
            last_try: None,
            retry: Duration::from_millis(DEFAULT_RETRY),
        }
    }

    fn next_request(&mut self) -> Result<()> {
        let mut list = List::new();
        if let Some(ref id) = self.last_event_id {
            list.append(&format!("Last-Event-ID: {}", id))?;
        }
        list.append("Accept: text/event-stream")?;
        self.handle.http_headers(list)?;
        self.handle.url(&self.url)?;
        self.handle.get(true)?;
        let mut current_line = &mut self.current_line;
        let mut transfer = self.handle.transfer();
        current_line.clear();
        transfer.write_function(|data| {
            current_line.push_str(str::from_utf8(data).unwrap());
            // Read data to the next newline.
            if current_line.find('\n').is_some() {
                Err(WriteError::Pause)
            } else {
                Ok(data.len())
            }
        })?;
        // TODO: Verify status code and Content-Type header.
        // TODO: This blocks. Use curl::multi instead...
        transfer.perform()?;
        Ok(())
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
    type Item = Result<Event>;

    fn next(&mut self) -> Option<Result<Event>> {
        //if self.transfer.is_none() {
            // We may have to wait for the next request.
            if let Some(last_try) = self.last_try {
                let elapsed = last_try.elapsed();
                if elapsed < self.retry {
                    ::std::thread::sleep(self.retry - elapsed);
                }
            }
            // Set here in case the request fails.
            self.last_try = Some(Instant::now());

            try_option!(self.next_request());
        //}

        let mut event = Event::new();
        let mut done = false;
        loop {
            if let Some(index) = self.current_line.find('\n') {
                done = false;
                // Split at index+1 to have the newline in `line`.
                // TODO: Use split_off here once that's stable.
                let (line, rest) = {
                    let (l, r) = self.current_line.split_at(index+1);
                    (l.to_string(), r.to_string())
                };
                self.current_line = rest;
                match parse_event_line(&line, &mut event) {
                    ParseResult::Next => (), // okay, just continue
                    ParseResult::Dispatch => {
                        return Some(Ok(event));
                    },
                    ParseResult::SetRetry(ref retry) => {
                        self.retry = *retry;
                    }
                }
            } else {
                try_option!(self.handle.unpause_write());
                if done { break; }
                done = true;
                // TODO: Maybe dispatch a partial event at the end of the stream?
            }
        }

        // EOF, retry after timeout
        self.last_try = Some(Instant::now());
        self.next()
    }
}

