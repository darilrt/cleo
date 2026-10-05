use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(msg) => write!(f, "Error: {}", msg),
            Error::Io(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Generic(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
