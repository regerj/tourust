use std::{fmt, io};

pub enum Error {
    IoError(io::Error),
    SynError(syn::Error),
    MissingSourceText,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => std::fmt::Display::fmt(err, f),
            Self::SynError(err) => std::fmt::Display::fmt(err, f),
            Self::MissingSourceText => write!(f, "Source text missing"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => std::fmt::Debug::fmt(err, f),
            Self::SynError(err) => std::fmt::Debug::fmt(err, f),
            Self::MissingSourceText => write!(f, "Source text missing"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Error::SynError(value)
    }
}
