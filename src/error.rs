use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Io(io::Error),
    Syn(syn::Error),
    Nvim(nvim_rs::error::CallError),
    Bat(bat::error::Error),
    Logger(flexi_logger::FlexiLoggerError),
    Translate(ansi_to_tui::Error),
    Utf8,
    NoWindow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => std::fmt::Display::fmt(err, f),
            Self::Syn(err) => std::fmt::Display::fmt(err, f),
            Self::Nvim(err) => std::fmt::Display::fmt(err, f),
            Self::Bat(err) => std::fmt::Display::fmt(err, f),
            Self::Logger(err) => std::fmt::Display::fmt(err, f),
            Self::Translate(err) => std::fmt::Display::fmt(err, f),
            Self::Utf8 => write!(f, "Invalid utf-8 could not be parsed"),
            Self::NoWindow => write!(f, "No valid window found"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => std::fmt::Debug::fmt(err, f),
            Self::Syn(err) => std::fmt::Debug::fmt(err, f),
            Self::Nvim(err) => std::fmt::Debug::fmt(err, f),
            Self::Bat(err) => std::fmt::Debug::fmt(err, f),
            Self::Logger(err) => std::fmt::Debug::fmt(err, f),
            Self::Translate(err) => std::fmt::Debug::fmt(err, f),
            Self::Utf8 => write!(f, "Invalid utf-8 could not be parsed"),
            Self::NoWindow => write!(f, "No valid window found"),
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

impl From<bat::error::Error> for Error {
    fn from(value: bat::error::Error) -> Self {
        Error::Bat(value)
    }
}

impl From<flexi_logger::FlexiLoggerError> for Error {
    fn from(value: flexi_logger::FlexiLoggerError) -> Self {
        Error::Logger(value)
    }
}

impl From<ansi_to_tui::Error> for Error {
    fn from(value: ansi_to_tui::Error) -> Self {
        Error::Translate(value)
    }
}
