use crate::{
    encoding::Template,
    judge::verdict::Verdict,
    types::{DataId, Result, TestMeta},
};

pub trait MetaEncoding<'a>: Sized {
    fn new(template: &Template, max_ignore: usize) -> Result<Self>;
    fn init(&mut self);
    fn ignore(&mut self, hash: &'a DataId);
    fn generate(&self) -> Result<String>;
    fn decode(message: Verdict) -> Result<TestMeta>;
}

pub trait DataEncoder<'a>: Sized {
    fn new(template: &Template, max_ignore: usize) -> Result<Self>;
    fn init(&mut self);
    fn push_ignore(&mut self, hash: &'a DataId);
    fn pop_ignore(&mut self);
    fn generate(&self, offset: usize) -> Result<String>;
}

pub trait DataDecoder: Sized {
    fn new() -> Self;
    fn init(&mut self, test: &TestMeta);
    fn append_message(&mut self, message: &str);
    fn clear(&mut self);
    fn decode(&mut self) -> Result<String>;
}
