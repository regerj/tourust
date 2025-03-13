use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Io(io::Error),
    Syn(syn::Error),
    Nvim(nvim_rs::error::CallError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => std::fmt::Display::fmt(err, f),
            Self::Syn(err) => std::fmt::Display::fmt(err, f),
            Self::Nvim(err) => std::fmt::Display::fmt(err, f),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => std::fmt::Debug::fmt(err, f),
            Self::Syn(err) => std::fmt::Debug::fmt(err, f),
            Self::Nvim(err) => std::fmt::Debug::fmt(err, f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Error::Syn(value)
    }
}

impl From<nvim_rs::error::CallError> for Error {
    fn from(value: nvim_rs::error::CallError) -> Self {
        Error::Nvim(value)
    }
}

impl From<Box<nvim_rs::error::CallError>> for Error {
    fn from(value: Box<nvim_rs::error::CallError>) -> Self {
        Error::Nvim(*value)
    }
}
