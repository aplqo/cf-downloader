use crate::{
    client::Verdict,
    encoding::Template,
    types::{DataId, Result, TestMeta},
};

pub trait MetaEncoding<'a>: Sized {
    fn new(template: &Template, max_ignore: usize) -> Result<Self>;
    fn ignore(&mut self, hash: &'a DataId);
    fn generate(&mut self) -> Result<&String>;
    fn decode(message: Verdict) -> Result<TestMeta>;
}

pub trait DataEncoder<'a>: Sized {
    fn new(template: &Template, max_ignore: usize) -> Result<Self>;
    fn push_ignore(&mut self, hash: &'a DataId);
    fn pop_ignore(&mut self);
    fn generate(&mut self, offset: usize) -> Result<&String>;
}

pub trait DataDecoder: Sized {
    fn new() -> Self;
    fn init(&mut self, test: &TestMeta);
    fn add_message(&mut self, offset: usize, message: &str);
    fn clear(&mut self);
    fn decode(&mut self) -> Result<String>;
}
