extern crate serde;

use crate::{
    encoding::{traits, utility::random, Template},
    types::{DataId, Result, BLOCK},
};
use handlebars::Handlebars;
use serde::Serialize;
use std::vec::Vec;

#[derive(Serialize)]
struct EncParam<'a> {
    random: u64,
    length: usize,
    offset: usize,
    ignore: Vec<&'a DataId>,
}
pub struct Encoder<'a> {
    result: String,
    param: EncParam<'a>,
    engine: Handlebars<'a>,
}

impl<'a> traits::DataEncoder<'a> for Encoder<'a> {
    fn new(template: &Template, max: usize) -> Result<Self> {
        let mut ret = Encoder {
            result: String::new(),
            param: EncParam {
                random: 0,
                length: BLOCK,
                offset: 0,
                ignore: Vec::new(),
            },
            engine: Handlebars::new(),
        };
        ret.param.ignore.reserve(max);
        ret.engine
            .register_template_string("code", template.content.as_str())?;
        Ok(ret)
    }
    fn init(&mut self) {
        self.param.random = random();
    }
    fn push_ignore(&mut self, hash: &'a DataId) {
        self.param.ignore.push(hash);
    }
    fn pop_ignore(&mut self) {
        self.param.ignore.pop();
    }
    fn generate(&mut self, offset: usize) -> Result<&String> {
        self.param.offset = offset;
        self.result = self.engine.render("code", &self.param)?;
        Ok(&self.result)
    }
}
