extern crate handlebars;

use handlebars::{RenderError, TemplateError};
use std::{
    boxed::Box, error::Error as StdError, fmt, num::ParseIntError, result::Result as StdResult,
};

#[derive(Debug)]
pub(super) struct Error(Box<Kind>);

#[derive(Debug)]
pub(super) enum Kind {
    ParseInt(&'static str, ParseIntError),
    Template(TemplateError),
    Rander(RenderError),
    Split(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Kind::ParseInt(name, err) => write!(f, "Error parse {}: {}", name, err.to_string()),
            Kind::Template(err) => write!(f, "Error parsing template: {}", err.to_string()),
            Kind::Rander(err) => write!(f, "Error generating code: {}", err.to_string()),
            Kind::Split(name) => write!(f, "Can't find {}", name),
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.0 {
            Kind::ParseInt(_, err) => Some(err.as_ref()),
            Kind::Template(err) => Some(&err),
            Kind::Rander(err) => Some(&err),
            Kind::Split(_) => None,
        }
    }
}
impl Error {
    pub(super) fn new(kind: Kind) -> Self {
        Error(Box::new(kind))
    }
}

pub(super) type Result<T> = StdResult<T, Error>;

pub(super) fn template_error(error: TemplateError) -> Error {
    Error::new(Kind::Template(error))
}
pub(super) fn rander_error(error: RenderError) -> Error {
    Error::new(Kind::Rander(error))
}
