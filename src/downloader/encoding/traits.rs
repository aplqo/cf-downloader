use crate::{
    encoding::Template,
    error::Error,
    judge::Verdict,
    types::{DataId, TestMeta},
};

pub trait MetaEncoding<'a>: Sized
where
    Self::Error: Error,
{
    type Error;
    fn new(template: &Template, max_ignore: usize) -> Result<Self, Self::Error>;
    fn init(&mut self);
    fn ignore(&mut self, hash: &'a DataId);
    fn generate(&self) -> Result<String, Self::Error>;
    fn decode(message: Verdict) -> Result<TestMeta, Self::Error>;
}

pub trait DataEncoder<'a>: Sized
where
    Self::Error: Error,
{
    type Error;
    fn new(template: &Template, max_ignore: usize) -> Result<Self, Self::Error>;
    fn init(&mut self);
    fn push_ignore(&mut self, hash: &'a DataId);
    fn pop_ignore(&mut self);
    fn generate(&self, offset: usize) -> Result<String, Self::Error>;
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
