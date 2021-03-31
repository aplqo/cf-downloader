extern crate reqwest;

mod error;
pub mod problem;
pub mod register;
mod retry;
mod search;
pub mod session;
pub mod submit;

pub use error::{Error, Result};

struct UtilityRegex {
    session: session::RegexSet,
    submit: submit::RegexSet,
}
impl UtilityRegex {
    fn new() -> Self {
        Self {
            session: session::RegexSet::new(),
            submit: submit::RegexSet::new(),
        }
    }
}

pub struct Session {
    client: reqwest::Client,
    pub handle: String,
    pub online: bool,
    ftaa: String,
    regex: UtilityRegex,
}

pub struct Verdict {
    pub(crate) input: Option<String>,
    pub(crate) output: String,
    pub(crate) answer: Option<String>,
}
