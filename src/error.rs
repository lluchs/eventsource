extern crate hyper;

use std::{error, fmt, io};
use hyper::header::ContentType;

/// Errors that can occur while reading events.
#[derive(Debug)]
pub enum Error {
    /// Some error from Hyper.
    Hyper(hyper::error::Error),
    /// HTTP request not successful.
    Http(hyper::status::StatusCode),
    /// IO error while reading a response body.
    Io(io::Error),
    /// Invalid or no Content-Type returned by the server.
    InvalidContentType(Option<ContentType>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Hyper(ref err) => write!(f, "Hyper error: {}", err),
            Error::Http(ref status) => write!(f, "HTTP request failed: {}", status),
            Error::Io(ref err) => write!(f, "I/O error: {}", err),
            Error::InvalidContentType(Some(ref ct)) => write!(f, "Invalid content type: {}", ct),
            Error::InvalidContentType(None) => write!(f, "No content type"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Hyper(ref err) => err.description(),
            Error::Http(ref status) => status.canonical_reason().unwrap_or("Unknown error"),
            Error::Io(ref err) => err.description(),
            Error::InvalidContentType(_) => "Content-Type returned by server must be text/event-stream",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Hyper(ref err) => Some(err),
            Error::Http(_) => None,
            Error::Io(ref err) => Some(err),
            Error::InvalidContentType(_) => None,
        }
    }
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Error {
        Error::Hyper(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

