extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DataId {
    pub hash: String,
    pub(crate) answer: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct TestMeta {
    pub data_id: DataId,
    pub size: usize,
    pub output_size: usize,
    pub compress_size: usize,
}

pub const BLOCK: usize = 500;

#[derive(Debug)]
pub struct Error {
    description: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}", self.description)
    }
}
impl std::error::Error for Error {}
impl Error {
    pub fn new(description: String) -> Box<Self> {
        Box::new(Error { description })
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
