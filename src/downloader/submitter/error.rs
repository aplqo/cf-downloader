extern crate tokio;

use crate::judge;
use std::{error::Error as StdError, fmt, result::Result as StdResult};
use tokio::task::JoinError;

#[derive(Debug, Clone, Copy)]
pub(super) enum Operate {
    BuildClient,
    Login,
    Submit,
    GetSubmission,
    Logout,
}
#[derive(Debug)]
pub(super) enum Kind {
    Join(JoinError),
    Judge(judge::Error),
}
#[derive(Debug)]
pub struct Error {
    operate: Operate,
    kind: Kind,
    handle: String,
}
pub type Result<T> = StdResult<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Kind::Join(x) => write!(f, "Error joining task using {}: {}", self.handle, x),
            Kind::Judge(handle, x) => {
                write!(
                    f,
                    "Error while {} using {}: {}",
                    self.operate, self.handle, x
                )
            }
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Kind::Join(x) => Some(x),
            Kind::Judge(x) => Some(x),
        }
    }
}
impl fmt::Display for Operate {
    fn fmt(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildClient => f.write_str("building client"),
            Self::Login => f.write_str("login"),
            Self::Submit => f.write_str("submitting code"),
            Self::GetSubmission => f.write_str("getting submission"),
            Self::Logout => f.write_str("logout"),
        }
    }
}
