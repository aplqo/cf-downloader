extern crate reqwest;

use crate::email;
use std::{boxed::Box, convert::Into, error::Error as StdError, fmt, result::Result as StdResult};

#[derive(Debug)]
pub(super) enum Kind {
    Builder(reqwest::Error),
    Network(reqwest::Error),
    CSRF(Box<Error>),
    API,
    Regex,
    Email(email::Error),
    TestCount(usize, usize),
}
#[derive(Debug)]
pub struct Error {
    kind: Kind,
    description: Option<String>,
}

pub type Result<T> = StdResult<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Builder(err) => write!(f, "Error building client: {}", err),
            Kind::Network(err) => write!(f, "Error sending request: {}", err),
            Kind::CSRF(x) => write!(f, "Error getting csrf token: {}", x),
            Kind::API => {
                write!(f, "API request failed")?;
                self.write_description(f)
            }
            Kind::Regex => {
                write!(f, "Regex not matched")?;
                self.write_description(f)
            }
            Kind::Email(err) => write!(f, "Email client: {}", err),
            Kind::TestCount(count, expect) => {
                write!(f, "Test count not match. Expected {} got {}", expect, count)
            }
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            Kind::Builder(x) | Kind::Network(x) => Some(x),
            Kind::CSRF(x) => Some(x.as_ref()),
            Kind::Email(e) => Some(e),
            Kind::API | Kind::Regex | Kind::TestCount(_, _) => None,
        }
    }
}
impl Error {
    pub(super) fn with_kind(kind: Kind) -> Self {
        Self {
            kind,
            description: None,
        }
    }
    pub(super) fn with_description<T: Into<String>>(kind: Kind, description: T) -> Self {
        Self {
            kind,
            description: Some(T::into(description)),
        }
    }
    fn write_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(d) = &self.description {
            write!(f, ": {}", d)
        } else {
            Ok(())
        }
    }
}

pub(super) fn network_error(err: reqwest::Error) -> Error {
    Error::with_kind(Kind::Network(err))
}
pub(super) fn regex_mismatch(description: Option<String>) -> Error {
    Error {
        kind: Kind::Regex,
        description,
    }
}
