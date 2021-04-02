extern crate serde;
extern crate serde_yaml;

use super::{Cache, SubmitKey};
use crate::judge::{problem::Problem, Verdict};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_reader, to_writer};
use std::{
    collections::HashMap,
    error::Error as StdError,
    fmt,
    io::{Read, Write},
};

#[derive(Debug)]
pub enum StoageError {
    Mismatch(Problem, Problem),
    Yaml(serde_yaml::Error),
}
impl fmt::Display for StoageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mismatch(expect, actual) => {
                write!(f, "Problem mismatch expected {} got {}", expect, actual)
            }
            Self::Yaml(e) => write!(f, "Error proess file: {}", e),
        }
    }
}
impl StdError for StoageError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Mismatch(_, _) => None,
            Self::Yaml(e) => Some(e),
        }
    }
}

#[derive(Serialize)]
struct SaveContent<'a> {
    problem: &'a Problem,
    content: &'a HashMap<SubmitKey, Verdict>,
}
#[derive(Deserialize)]
struct LoadContent {
    problem: Problem,
    content: HashMap<SubmitKey, Verdict>,
}

impl<'a> Cache<'a> {
    pub fn save<W: Write>(&self, wr: W) -> Result<(), StoageError> {
        to_writer(
            wr,
            &SaveContent {
                problem: &self.problem,
                content: &self.cache,
            },
        )
        .map_err(StoageError::Yaml)
    }
    pub fn load<R: Read>(&mut self, rdr: R) -> Result<(), StoageError> {
        let val: LoadContent = from_reader(rdr).map_err(StoageError::Yaml)?;
        if self.problem.as_ref() != &val.problem {
            Err(StoageError::Mismatch(
                self.problem.as_ref().clone(),
                val.problem,
            ))
        } else {
            self.cache = val.content;
            Ok(())
        }
    }
}
