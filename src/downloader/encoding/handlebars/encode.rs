extern crate handlebars;
extern crate serde;

use super::error::{rander_error, template_error, Error, Result};
use crate::{
    encoding::{traits, Template},
    random::random_standard,
    types::{DataId, BLOCK},
};
use handlebars::Handlebars;
use serde::Serialize;
use std::vec::Vec;

#[derive(Serialize)]
struct EncParam<'a, 'b> {
    random: u64,
    length: usize,
    offset: usize,
    ignore: &'b Vec<&'a DataId>,
}
pub struct Encoder<'a> {
    random: u64,
    length: usize,
    ignore: Vec<&'a DataId>,
    engine: Handlebars<'a>,
}

impl<'a> traits::DataEncoder<'a, Error> for Encoder<'a> {
    fn new(template: &Template, max: usize) -> Result<Self> {
        let mut ret = Encoder {
            random: 0,
            length: BLOCK,
            ignore: Vec::with_capacity(max),
            engine: Handlebars::new(),
        };
        ret.engine
            .register_template_string("code", template.content.as_str())
            .map_err(template_error)?;
        Ok(ret)
    }
    fn init(&mut self) {
        self.random = random_standard();
    }
    fn push_ignore<'b: 'a>(&mut self, hash: &'b DataId) {
        self.ignore.push(hash);
    }
    fn pop_ignore(&mut self) {
        self.ignore.pop();
    }
    fn generate(&self, offset: usize) -> Result<String> {
        self.engine
            .render(
                "code",
                &EncParam {
                    random: self.random,
                    length: self.length,
                    offset,
                    ignore: &self.ignore,
                },
            )
            .map_err(rander_error)
    }
}
