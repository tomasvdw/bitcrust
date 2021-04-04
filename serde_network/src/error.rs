use std::{self, error, io, str};
use std::fmt::{self, Display};
use serde;

#[derive(Debug)]
pub enum Error {
    EndOfBufferError,
    IOError(io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;




impl error::Error for Error {
    fn description(&self) -> &str {
        let x = match *self {
            Error::EndOfBufferError => "Unexpected end of buffer",
            Error::IOError(ref _io) => "io error", // TODO need to get the error message out of io
        };
        return x;
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(_desc: T) -> Error {
        Error::EndOfBufferError
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Error::EndOfBufferError
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Serde_network error")
    }
}


impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}
