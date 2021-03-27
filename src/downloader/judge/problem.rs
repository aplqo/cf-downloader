extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Type {
    Contest,
    Gym,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub(super) submit_url: String,
    pub(super) status_url: String,
    pub source: Type,
    pub contest: String,
    pub id: String,
}
impl PartialEq for Problem {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.contest == other.contest && self.id == other.id
    }
}
impl Problem {
    pub fn new(source: Type, contest: String, id: String) -> Self {
        Problem {
            submit_url: format!("https://codeforces.com/contest/{}/submit", contest),
            status_url: format!("https://codeforces.com/contest/{}/status", contest),
            source,
            contest,
            id,
        }
    }
}

pub(super) fn get_problem_url(_source: Type, contest: &str, id: &str) -> String {
    format!("https://codeforces.com/contest/{}/problem/{}", contest, id)
}
