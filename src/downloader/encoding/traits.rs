use crate::{
    encoding::Template,
    error::Error,
    judge::Verdict,
    types::{DataId, TestMeta},
};

pub trait MetaEncoding<'a, Err: Error>: Sized {
    fn new(template: &Template, max_ignore: usize) -> Result<Self, Err>;
    fn init(&mut self);
    fn ignore<'b: 'a>(&mut self, hash: &'b DataId);
    fn generate(&self) -> Result<String, Err>;
    fn decode(message: Verdict) -> Result<TestMeta, Err>;
}

pub trait DataEncoder<'a, Err: Error>: Sized {
    fn new(template: &Template, max_ignore: usize) -> Result<Self, Err>;
    fn init(&mut self);
    fn push_ignore<'b: 'a>(&mut self, hash: &'b DataId);
    fn pop_ignore(&mut self);
    fn generate(&self, offset: usize) -> Result<String, Err>;
}

pub trait DataDecoder: Sized
where
    Self::Error: Error,
{
    type Error;
    fn new() -> Self;
    fn init(&mut self, test: &TestMeta);
    fn append_message(&mut self, message: &str);
    fn clear(&mut self);
    fn decode(&mut self) -> Result<String, Self::Error>;
}
