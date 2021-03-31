extern crate serde;

use crate::{
    judge::{problem::Problem, Verdict},
    submitter::Submitter,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SubmitKey {
    pub test: usize,
    pub time: usize,
}
impl fmt::Display for SubmitKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test {} #{}", self.test, self.time)
    }
}

pub struct Cache<'a> {
    problem: &'a Problem,
    submitter: &'a mut Submitter,
    cache: HashMap<SubmitKey, Verdict>,
}

impl<'a> Cache<'a> {
    pub fn new(problem: &'a Problem, submitter: &'a mut Submitter) -> Self {
        Self {
            problem,
            submitter,
            cache: HashMap::new(),
        }
    }
    pub fn flush(&mut self) {
        self.cache.clear();
    }
}

pub mod storage;
pub mod submit;
