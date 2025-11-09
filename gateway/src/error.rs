use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Config(String),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Config(err) => write!(f, "{}", err),
            Error::Io(err) => write!(f, "{}", err),
        }
    }
}

impl From<envconfig::Error> for Error {
    fn from(err: envconfig::Error) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
