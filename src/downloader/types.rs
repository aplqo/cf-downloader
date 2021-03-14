extern crate serde;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct TestMeta {
    pub hash: String,
    pub size: usize,
    pub output_size: usize,
    pub(crate) answer: Option<String>,
}
pub struct Template<'a> {
    pub language: &'a String,
    pub content: &'a String,
}
pub struct Error {
    pub description: String,
}
pub const BLOCK: usize = 500;
