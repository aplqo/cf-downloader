extern crate handlebars;
extern crate serde;

use super::error::{rander_error, template_error, Error, Result};
use crate::{
    encoding::{traits, Template},
    judge::Verdict,
    random::random_standard,
    types::{DataId, TestMeta},
};
use handlebars::Handlebars;
use serde::Serialize;
use std::str::SplitWhitespace;

#[derive(Serialize)]
struct MetaParam<'a, 'b> {
    random: u64,
    ignore: &'b Vec<&'a DataId>,
}
pub struct Meta<'a> {
    random: u64,
    ignore: Vec<&'a DataId>,
    engine: Handlebars<'a>,
}

fn split_error(name: &'static str) -> Error {
    Error::Split(name)
}
fn next_usize(split: &mut SplitWhitespace, name: &'static str) -> Result<usize> {
    split
        .next()
        .ok_or_else(|| split_error(name))?
        .parse()
        .map_err(|x| Error::ParseInt(name, x))
}

impl<'a> traits::MetaEncoding<'a, Error> for Meta<'a> {
    fn new(template: &Template, max: usize) -> Result<Self> {
        let mut ret = Meta {
            random: 0,
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
    fn ignore<'b: 'a>(&mut self, hash: &'b DataId) {
        self.ignore.push(hash);
    }
    fn generate(&self) -> Result<String> {
        self.engine
            .render(
                "code",
                &MetaParam {
                    random: self.random,
                    ignore: &self.ignore,
                },
            )
            .map_err(rander_error)
    }
    fn decode(message: Verdict) -> Result<TestMeta> {
        let mut p = message.output.split_whitespace();
        Ok(TestMeta {
            size: next_usize(&mut p, "size")?,
            output_size: next_usize(&mut p, "output_size")?,
            compress_size: next_usize(&mut p, "compress_size")?,
            data_id: DataId {
                hash: p.next().ok_or_else(|| split_error("hash"))?.to_string(),
                answer: message.answer,
            },
            input: message.input,
        })
    }
}
