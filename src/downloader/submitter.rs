pub mod error;
mod list;
pub mod session;
pub mod submit;

pub use error::{Error, Result};

pub struct Submitter {
    session: std::vec::Vec<crate::judge::Session>,
    list: list::AccountList,
}
impl Submitter {
    pub fn new() -> Self {
        Submitter {
            session: Vec::new(),
            list: list::AccountList::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.session.is_empty()
    }
}
impl Default for Submitter {
    fn default() -> Self {
        Self::new()
    }
}
