extern crate base64;
extern crate flate2;

use crate::{
    encoding::traits::DataDecoder,
    types::{Result, TestMeta},
};
use flate2::read::GzDecoder;
use std::{io::Read, vec::Vec};

pub struct Decoder {
    buffer: Vec<u8>,
    decoded: Vec<u8>,
    output_size: usize,
}

impl DataDecoder for Decoder {
    fn new() -> Self {
        Decoder {
            buffer: Vec::new(),
            decoded: Vec::new(),
            output_size: 0,
        }
    }
    fn init(&mut self, data: &TestMeta) {
        self.buffer.reserve(data.output_size);
        self.decoded.reserve(data.compress_size);
        self.output_size = data.size;
    }
    fn append_message(&mut self, message: &str) {
        self.buffer.extend_from_slice(message.as_bytes());
    }
    fn clear(&mut self) {
        self.buffer.clear();
        self.decoded.clear();
    }
    fn decode(&mut self) -> Result<String> {
        base64::decode_config_buf(&self.buffer, base64::STANDARD, &mut self.decoded)?;
        let mut ret = String::new();
        ret.reserve(self.output_size);
        GzDecoder::new(self.decoded.as_slice()).read_to_string(&mut ret)?;
        Ok(ret)
    }
}
