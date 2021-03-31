extern crate base64;
extern crate flate2;

use crate::{encoding::traits::DataDecoder, types::TestMeta};
use base64::{decode_config_buf, DecodeError};
use flate2::read::GzDecoder;
use std::{
    error::Error as StdError,
    fmt,
    io::{self, Read},
    vec::Vec,
};

#[derive(Debug)]
pub enum Error {
    Decode(DecodeError),
    Decompress(io::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Decode(dec) => write!(f, "base64: {}", dec),
            Error::Decompress(err) => write!(f, "gzip: {}", err),
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Decode(x) => Some(x),
            Error::Decompress(x) => Some(x),
        }
    }
}
type Result<T> = std::result::Result<T, Error>;

pub struct Decoder {
    buffer: Vec<u8>,
    decoded: Vec<u8>,
    output_size: usize,
}

impl DataDecoder for Decoder {
    type Error = Error;

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
        decode_config_buf(&self.buffer, base64::STANDARD, &mut self.decoded)
            .map_err(|x| Error::Decode(x))?;
        let mut ret = String::new();
        ret.reserve(self.output_size);
        GzDecoder::new(self.decoded.as_slice())
            .read_to_string(&mut ret)
            .map_err(|x| Error::Decompress(x))?;
        Ok(ret)
    }
}
