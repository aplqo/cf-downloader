use crate::client::Submission;
use crate::types::{Error, Template, TestMeta};

pub trait MetaEncoding {
    fn new(template: &Template, max_ignore: usize) -> Self;
    fn ignore_input(&mut self, hash: &String);
    fn generate_encoder(&self) -> &String;
    fn decode(message: Submission) -> Result<TestMeta, Error>;
}

pub trait DataEncoder {
    fn new(template: &Template, data: &[TestMeta]) -> Self;
    fn ignore_input(&mut self, pos: usize);
    fn enable_input(&mut self, pos: usize);
    fn generate_enoder(&self) -> &String;
}

pub trait DataDecoder {
    fn new(test: &TestMeta) -> Self;
    fn add_message(&mut self, offset: i32, message: &String);
    fn clear(&mut self);
    fn decode(&mut self) -> Result<String, Error>;
}
