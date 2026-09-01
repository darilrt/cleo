use std::io;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Io(io::Error),
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
