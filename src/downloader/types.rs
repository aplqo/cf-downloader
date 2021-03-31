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
    pub input: Option<String>,
    pub size: usize,
    pub output_size: usize,
    pub compress_size: usize,
}

pub const BLOCK: usize = 500;
