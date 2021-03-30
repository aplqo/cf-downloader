extern crate handlebars;
extern crate serde;

use super::error::{rander_error, template_error, Error, Kind, Result};
use crate::{
    encoding::{traits, Template},
    judge::verdict::Verdict,
    random::random_standard,
    types::{DataId, TestMeta},
};
use handlebars::Handlebars;
use serde::Serialize;
use std::str::SplitWhitespace;

#[derive(Serialize)]
struct MetaParam<'a> {
    random: u64,
    ignore: Vec<&'a DataId>,
}
pub struct Meta<'a> {
    param: MetaParam<'a>,
    engine: Handlebars<'a>,
}

fn split_error(name: &str) -> Error {
    Error::new(Kind::Split(name))
}
fn next_usize(split: &mut SplitWhitespace, name: &'static str) -> Result<usize> {
    split
        .next()
        .ok_or_else(|| split_error(name))?
        .parse()
        .map_err(|x| Error::new(Kind::ParseInt(name, x)))
}

impl<'a> traits::MetaEncoding<'a> for Meta<'a> {
    fn new(template: &Template, max: usize) -> Result<Self> {
        let mut ret = Meta {
            param: MetaParam {
                random: 0,
                ignore: Vec::with_capacity(max),
            },
            engine: Handlebars::new(),
        };
        ret.engine
            .register_template_string("code", template.content.as_str())
            .map_err(template_error)?;
        Ok(ret)
    }
    fn init(&mut self) {
        self.param.random = random_standard();
    }
    fn ignore(&mut self, hash: &'a DataId) {
        self.param.ignore.push(hash);
    }
    fn generate(&self) -> Result<String> {
        self.engine
            .render("code", &self.param)
            .map_err(rander_error)
    }
    fn decode(message: Verdict) -> Result<TestMeta> {
        let mut p = message.output.split_whitespace();
        Ok(TestMeta {
            size: next_usize(p, "size")?,
            output_size: next_usize(p, "output_size")?,
            compress_size: next_usize(p, "compress_size")?,
            data_id: DataId {
                hash: p.next().ok_or_else(|| split_error("hash"))?.to_string(),
                answer: message.answer,
            },
            input: message.input,
        })
    }
}
