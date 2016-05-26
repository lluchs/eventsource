#[macro_use] extern crate hyper;

use std::io::{BufRead, BufReader};
use std::time::Duration;
use hyper::client::{Client as HyperClient};
use hyper::client::response::Response;
use hyper::header::Headers;
use hyper::Url;

const DEFAULT_RETRY: u64 = 5000;

header! { (LastEventID, "Last-Event-ID") => [String] }

pub struct Client {
    hc: HyperClient,
    reader: Option<BufReader<Response>>,
    url: Url,
    last_event_id: Option<String>,
    retry: u64, // reconnection time in milliseconds
}

#[derive(Debug)]
pub struct Event {
    pub id: Option<String>,
    pub event_type: Option<String>,
    pub data: String,
}

enum ParseResult {
    Next,
    Dispatch,
}

impl Client {
    pub fn new(url: Url) -> Client {
        Client {
            hc: HyperClient::new(),
            reader: None,
            url: url,
            last_event_id: None,
            retry: DEFAULT_RETRY,
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
                        self.retry = retry;
                    }
                },
                _ => () // ignored
            }

            ParseResult::Next
        }
    }
}

// Iterate over the client to get events.
impl Iterator for Client {
    // TODO: Result<Event>
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if self.reader.is_none() {
            let r = BufReader::new(self.next_request().unwrap());
            self.reader = Some(r);
        }
        let mut event = Event::new();
        let mut line = String::new();

        // We can't have a mutable reference to the reader because of the &mut self call below.
        while self.reader.as_mut().unwrap().read_line(&mut line).unwrap() > 0 {
            match self.parse_event_line(&line, &mut event) {
                ParseResult::Dispatch => return Some(event),
                ParseResult::Next => (),
            }
            line.clear();
        }
        // EOF, retry after timeout
        self.reader = None;
        std::thread::sleep(Duration::from_millis(self.retry));
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
