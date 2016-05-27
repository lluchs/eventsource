extern crate hyper;

use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
    // Some error from Hyper.
    Hyper(hyper::error::Error),
    // HTTP request not successful.
    Http(hyper::status::StatusCode),
    // IO error while reading a response body.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Hyper(ref err) => write!(f, "Hyper error: {}", err),
            Error::Http(ref status) => write!(f, "HTTP request failed: {}", status),
            Error::Io(ref err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Hyper(ref err) => err.description(),
            Error::Http(ref status) => status.canonical_reason().unwrap_or("Unknown error"),
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Hyper(ref err) => Some(err),
            Error::Http(_) => None,
            Error::Io(ref err) => Some(err),
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

