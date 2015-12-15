use std::fmt;
use std::io;
use std::error::Error;
use std::result;
use walkdir;
use mustache;
use chrono;

pub type Result<T> = result::Result<T, GoethiteError>;

pub enum GoethiteError {
    Io(io::Error),
    NotFound(String),
    InvalidConfig,
    Traverse,
    InvalidFrontMatter(String),
    Other,
    Template(mustache::encoder::Error),
    MissingLayout(String),
    InvalidDate(chrono::format::ParseError),
}

impl fmt::Display for GoethiteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GoethiteError::Io(ref err)                  => write!(f, "IO Error: {}", err),
            GoethiteError::NotFound(ref file)           => write!(f, "File Not Found: {}", file),
            GoethiteError::InvalidConfig                => write!(f, "Invalid config file"),
            GoethiteError::Traverse                     => write!(f, "Error while traversing source directory"),
            GoethiteError::InvalidFrontMatter(ref file) => write!(f, "Invalid front matter for: {}", file),
            GoethiteError::Other                        => write!(f, "General error occurred"),
            GoethiteError::Template(ref err)            => write!(f, "Template Error: {:?}", err),
            GoethiteError::MissingLayout(ref layout)    => write!(f, "Layout not found: {}", layout),
            GoethiteError::InvalidDate(ref err)         => write!(f, "Invalid date: {}", err),
        }
    }
}

impl From<io::Error> for GoethiteError {
    fn from(err: io::Error) -> GoethiteError {
        GoethiteError::Io(err)
    }
}

impl From<walkdir::Error> for GoethiteError {
    fn from(_: walkdir::Error) -> GoethiteError {
        GoethiteError::Traverse
    }
}

impl From<mustache::encoder::Error> for GoethiteError {
    fn from(err: mustache::encoder::Error) -> GoethiteError {
        GoethiteError::Template(err)
    }
}

impl From<chrono::format::ParseError> for GoethiteError {
    fn from(err: chrono::format::ParseError) -> GoethiteError {
        GoethiteError::InvalidDate(err)
    }
}
