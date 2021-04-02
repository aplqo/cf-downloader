use crate::{cache::Cache, judge::problem::Problem, submitter::Submitter, types::TestMeta};
use std::{rc::Rc, vec::Vec};

pub mod data;
pub mod meta;
pub mod meta_storage;

pub struct Downloader<'a> {
    problem: Rc<Problem>,
    data: Vec<TestMeta>,
    pub cache: Cache<'a>,
}

impl<'a> Downloader<'a> {
    pub fn new(problem: Problem, submitter: &'a mut Submitter) -> Self {
        let r = Rc::from(problem);
        Self {
            problem: r.clone(),
            data: Vec::new(),
            cache: Cache::new(r, submitter),
        }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
