extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Verdict {
    pub(crate) input: Option<String>,
    pub(crate) output: String,
    pub(crate) answer: Option<String>,
}
