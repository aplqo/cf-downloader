extern crate serde;

use super::{
    error::{network_error, Result},
    Session,
};
use serde::{Deserialize, Serialize};
use std::fmt;

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
impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Contest-{}{}", self.contest, self.id)
    }
}

fn get_problem_url(_source: Type, contest: &str, id: &str) -> String {
    format!("https://codeforces.com/contest/{}/problem/{}", contest, id)
}

impl Session {
    pub async fn check_exist(&self, source: Type, contest: &str, id: &str) -> Result<bool> {
        let url = get_problem_url(source, contest, id);
        Ok(url
            == self
                .client
                .get(&url)
                .send()
                .await
                .map_err(network_error)?
                .error_for_status()
                .map_err(network_error)?
                .url()
                .as_str())
    }
}
