use std::fmt;
use std::time::Duration;

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

/// Possible results from parsing a single event-stream line.
#[derive(Debug, PartialEq)]
pub enum ParseResult {
    /// Line parsed successfully, but the event is not complete yet.
    Next,
    /// The event is complete now. Pass a new (empty) event for the next call.
    Dispatch,
    /// Set retry time.
    SetRetry(Duration),
}

/// Parse a single line of an event-stream.
///
/// The line may end with a newline.
///
/// You will have to call this function multiple times until it returns `ParseResult::Dispatch`.
/// Make sure to clear the event struct for the next line, then.
///
/// To handle the `Last-Event-ID` header, check the `id` field for each finished event.
///
/// # Examples
///
/// ```
/// # use eventsource::event::{Event, ParseResult, parse_event_line};
/// let mut event = Event::new();
/// assert_eq!(parse_event_line("id: 42", &mut event), ParseResult::Next);
/// assert_eq!(parse_event_line("data: foobar", &mut event), ParseResult::Next);
/// assert_eq!(parse_event_line("", &mut event), ParseResult::Dispatch);
/// // The event is finished now.
/// assert_eq!(event.id, Some("42".into()));
/// assert_eq!(event.data, "foobar\n");
/// // Now clear and continue.
/// event.clear();
/// // ...
/// ```
pub fn parse_event_line(line: &str, event: &mut Event) -> ParseResult {
    let line = line.trim_right_matches(|c| c == '\r' || c == '\n');
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
            "id" => { event.id = Some(value.to_string()); }
            "retry" => {
                if let Ok(retry) = value.parse::<u64>() {
                    return ParseResult::SetRetry(Duration::from_millis(retry));
                }
            },
            _ => () // ignored
        }

        ParseResult::Next
    }
}

impl Event {
    /// Creates an empty event.
    pub fn new() -> Event {
        Event {
            id: None,
            event_type: None,
            data: "".to_string(),
        }
    }

    /// Returns `true` if the event is empty.
    ///
    /// An event is empty if it has no id or event type and its data field is empty.
    pub fn is_empty(&self) -> bool {
        self.id.is_none() && self.event_type.is_none() && self.data.is_empty()
    }

    /// Makes the event empty.
    pub fn clear(&mut self) {
        self.id = None;
        self.event_type = None;
        self.data.clear();
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
