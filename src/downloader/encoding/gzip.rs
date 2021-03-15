extern crate base64;
extern crate flate2;

use crate::{
    encoding::traits,
    types::{Result, TestMeta},
};
use flate2::Decompress;
use std::vec::Vec;

pub struct Decoder {
    buffer: Vec<u8>,
    decoded: Vec<u8>,
    decompress: Decompress,
}

impl traits::DataDecoder for Decoder {
    fn new() -> Self {
        Decoder {
            buffer: Vec::new(),
            decoded: Vec::new(),
            decompress: Decompress::new(true),
        }
    }
    fn init(&mut self, data: &TestMeta) {
        self.buffer.resize(data.output_size, 0);
        self.decoded.reserve(data.compress_size);
    }
    fn add_message(&mut self, offset: usize, message: &str) {
        self.buffer[offset..offset + message.len()].copy_from_slice(message.as_bytes());
    }
    fn clear(&mut self) {
        self.buffer.resize(0, 0);
    }
    fn decode(&mut self) -> Result<String> {
        let mut ret = String::new();
        base64::decode_config_buf(&self.buffer, base64::STANDARD, &mut self.decoded)?;
        unsafe {
            self.decompress.decompress_vec(
                self.decoded.as_slice(),
                ret.as_mut_vec(),
                flate2::FlushDecompress::Finish,
            )?;
        };
        Ok(ret)
    }
}
