extern crate serde;

use crate::{
    client::verdict::Verdict,
    encoding::{traits, utility::random, Template},
    types::{DataId, Result, TestMeta},
};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct MetaParam<'a> {
    random: u64,
    ignore: Vec<&'a DataId>,
}
pub struct Meta<'a> {
    param: MetaParam<'a>,
    engine: Handlebars<'a>,
}

impl<'a> traits::MetaEncoding<'a> for Meta<'a> {
    fn new(template: &Template, max: usize) -> Result<Self> {
        let mut ret = Meta {
            param: MetaParam {
                random: 0,
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
    fn ignore(&mut self, hash: &'a DataId) {
        self.param.ignore.push(hash);
    }
    fn generate(&self) -> Result<String> {
        Ok(self.engine.render("code", &self.param)?)
    }
    fn decode(message: Verdict) -> Result<TestMeta> {
        let mut p = message.output.split_whitespace();
        Ok(TestMeta {
            size: p.next().ok_or("Can't find size")?.parse()?,
            output_size: p.next().ok_or("Can't find output size")?.parse()?,
            compress_size: p.next().ok_or("Can't find compressed size")?.parse()?,
            data_id: DataId {
                hash: p.next().ok_or("Can't get input hash")?.to_string(),
                answer: message.answer,
            },
            input: message.input,
        })
    }
}
