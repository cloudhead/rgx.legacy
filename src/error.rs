use std::error;
use std::fmt;
use std::io;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoAdaptersFound,
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdaptersFound => write!(f, "a suitable graphics adapter was not found"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Self::NoAdaptersFound => "a suitable graphics adapter was not found",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
