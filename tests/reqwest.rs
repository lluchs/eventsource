extern crate eventsource;
extern crate reqwest;

use eventsource::reqwest::{Client, Error, ErrorKind};
use reqwest::Url;
use std::time::Duration;

use server::Server;
mod server;

fn server() -> Server {
    let s = Server::new();
    s.receive("\
GET / HTTP/1.1\r\n\
Host: 127.0.0.1:$PORT\r\n\
User-Agent: reqwest/0.8.0\r\n\
Accept: text/event-stream\r\n\
Accept-Encoding: gzip\r\n\
\r\n");
    return s;
}

#[test]
fn simple_events() {
    let s = server();
    s.send("HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream\r\n\
\r\n\
id: 42\r\n\
event: foo\r\n\
data: bar\r\n\
\r\n\
event: bar\n\
: comment\n\
data: baz\n\
\n");

    println!("url: {}", s.url("/"));
    let mut client = Client::new(Url::parse(&s.url("/")).unwrap());

    let event = client.next().unwrap().unwrap();
    assert_eq!(event.id, Some("42".into()));
    assert_eq!(event.event_type, Some("foo".into()));
    assert_eq!(event.data, "bar\n");

    let event = client.next().unwrap().unwrap();
    assert_eq!(event.id, None);
    assert_eq!(event.event_type, Some("bar".into()));
    assert_eq!(event.data, "baz\n");
}

#[test]
fn retry() {
    let s = server();
    s.send("HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream\r\n\
\r\n\
retry: 42\r\n\
data: bar\r\n\
\r\n");

    println!("url: {}", s.url("/"));
    let mut client = Client::new(Url::parse(&s.url("/")).unwrap());
    let event = client.next().unwrap().unwrap();
    assert_eq!(event.data, "bar\n");
    assert_eq!(client.retry, Duration::from_millis(42));
}

#[test]
fn missing_content_type() {
    let s = server();
    s.send("HTTP/1.1 200 OK\r\n\
\r\n\
data: bar\r\n\
\r\n");

    let mut client = Client::new(Url::parse(&s.url("/")).unwrap());
    match client.next().unwrap() {
        Err(Error(ErrorKind::NoContentType, _)) => assert!(true),
        _ => assert!(false, "NoContentType error expected"),
    }
}

#[test]
fn invalid_content_type() {
    let s = server();
    s.send("HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
\r\n\
data: bar\r\n\
\r\n");

    let mut client = Client::new(Url::parse(&s.url("/")).unwrap());
    match client.next().unwrap() {
        Err(Error(ErrorKind::InvalidContentType(_), _)) => assert!(true),
        _ => assert!(false, "InvalidContentType error expected"),
    }
}

#[test]
fn content_type_with_mime_parameter() {
    let s = server();
    s.send("HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream;charset=utf8\r\n\
\r\n\
data: bar\r\n\
\r\n");

    let mut client = Client::new(Url::parse(&s.url("/")).unwrap());
    let event = client.next().unwrap().expect("MIME parameter should be ignored");
    assert_eq!(event.data, "bar\n");
}
