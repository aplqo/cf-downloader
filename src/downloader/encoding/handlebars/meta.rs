extern crate serde;

use crate::{
    client::Verdict,
    encoding::{traits, Template},
    types::{DataId, Result, TestMeta},
};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct MetaParam<'a> {
    ignore: Vec<&'a DataId>,
}
pub struct Meta<'a> {
    result: String,
    param: MetaParam<'a>,
    engine: Handlebars<'a>,
}

impl<'a> traits::MetaEncoding<'a> for Meta<'a> {
    fn new(template: &Template, max: usize) -> Result<Self> {
        let mut ret = Meta {
            result: String::new(),
            param: MetaParam { ignore: Vec::new() },
            engine: Handlebars::new(),
        };
        ret.param.ignore.reserve(max);
        ret.engine
            .register_template_string("code", template.content.as_str())?;
        Ok(ret)
    }
    fn ignore(&mut self, hash: &'a DataId) {
        self.param.ignore.push(hash);
    }
    fn generate(&mut self) -> Result<&String> {
        self.result = self.engine.render("code", &self.param)?;
        Ok(&self.result)
    }
    fn decode(message: Verdict) -> Result<TestMeta> {
        let mut p = message.output.split_whitespace();
        Ok(TestMeta {
            size: p.nth(0).ok_or("Can't find size")?.parse()?,
            output_size: p.nth(0).ok_or("Can't find output size")?.parse()?,
            compress_size: p.nth(0).ok_or("Can't find compressed size")?.parse()?,
            data_id: DataId {
                hash: p.nth(0).ok_or("Can't get input hash")?.to_string(),
                answer: message.answer,
            },
            input: message.input,
        })
    }
}
