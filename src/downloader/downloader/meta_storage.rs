extern crate serde;
extern crate serde_yaml;

use super::Downloader;
use crate::{judge::problem::Problem, types::TestMeta};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_reader, to_writer};
use std::{
    error::Error as StdError,
    fmt,
    io::{Read, Write},
};

#[derive(Debug)]
pub enum Error {
    Mismatch(Problem, Problem),
    Yaml(serde_yaml::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mismatch(expect, actual) => {
                write!(f, "Problem mismatch, expect {} read {}", expect, actual)
            }
            Self::Yaml(err) => write!(f, "Error processing file: {}", err),
        }
    }
}
impl StdError for Error {
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
    data: &'a Vec<TestMeta>,
}
#[derive(Deserialize)]
struct LoadContent {
    problem: Problem,
    data: Vec<TestMeta>,
}

impl<'a> Downloader<'a> {
    pub fn load_meta<R: Read>(&mut self, rdr: R) -> Result<(), Error> {
        let lst: LoadContent = from_reader(rdr).map_err(|e| Error::Yaml(e))?;
        if &lst.problem != self.problem.as_ref() {
            Err(Error::Mismatch(self.problem.as_ref().clone(), lst.problem))
        } else {
            self.data = lst.data;
            Ok(())
        }
    }
    pub fn save_meta<W: Write>(&self, wdr: W) -> Result<(), Error> {
        to_writer(
            wdr,
            &SaveContent {
                problem: &self.problem,
                data: &self.data,
            },
        )
        .map_err(|x| Error::Yaml(x))
    }
}
