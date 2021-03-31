extern crate handlebars;

use handlebars::{RenderError, TemplateError};
use std::{error::Error as StdError, fmt, num::ParseIntError, result::Result as StdResult};

#[derive(Debug)]
pub(super) enum Error {
    ParseInt(&'static str, ParseIntError),
    Template(TemplateError),
    Rander(RenderError),
    Split(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseInt(name, err) => write!(f, "Error parse {}: {}", name, err.to_string()),
            Error::Template(err) => write!(f, "Error parsing template: {}", err.to_string()),
            Error::Rander(err) => write!(f, "Error generating code: {}", err.to_string()),
            Error::Split(name) => write!(f, "Can't find {}", name),
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ParseInt(_, err) => Some(err.as_ref()),
            Error::Template(err) => Some(&err),
            Error::Rander(err) => Some(&err),
            Error::Split(_) => None,
        }
    }
}

pub(super) type Result<T> = StdResult<T, Error>;

pub(super) fn template_error(error: TemplateError) -> Error {
    Error::Template(error)
}
pub(super) fn rander_error(error: RenderError) -> Error {
    Error::Rander(error)
}
