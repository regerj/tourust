use std::{fmt, io};

pub enum Error {
    IoError(io::Error),
    SynError(syn::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => std::fmt::Display::fmt(err, f),
            Self::SynError(err) => std::fmt::Display::fmt(err, f),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => std::fmt::Debug::fmt(err, f),
            Self::SynError(err) => std::fmt::Debug::fmt(err, f),
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
