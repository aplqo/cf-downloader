extern crate futures;

use super::Downloader;
use crate::{
    cache::{self, submit::Handle, Cache, SubmitKey},
    encoding::{DataDecoder, DataEncoder, Template},
    error::Error as ErrType,
    types::BLOCK,
};
use futures::future::join_all;
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error<EG: ErrType, ED: ErrType> {
    Build(EG),
    Submit(cache::submit::Error<EG>),
    Decode(usize, ED),
}
impl<EG: ErrType, ED: ErrType> fmt::Display for Error<ED, EG> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(e) => write!(f, "Error building template: {}", e),
            Self::Submit(e) => write!(f, "Error get mesage: {}", e),
            Self::Decode(id, e) => write!(f, "Error decode test {} message: {}", id, e),
        }
    }
}
impl<ED: ErrType, EG: ErrType> StdError for Error<ED, EG> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Build(e) => Some(e),
            Self::Submit(e) => Some(e),
            Self::Decode(_, e) => Some(e),
        }
    }
}
pub enum DataResult<EG: ErrType, ED: ErrType> {
    Build(Error<EG, ED>),
    Result(Vec<Result<String, Error<EG, ED>>>),
}

impl<'a> Downloader<'a> {
    async fn fetch<'b, 'c, Enc: DataEncoder<'b, Err>, Err: ErrType>(
        &'c mut self,
        template: &Template,
        begin: usize,
        end: usize,
    ) -> Result<Vec<Vec<Handle<Err>>>, Err>
    where
        'a: 'c,
        'c: 'b,
    {
        let mut encoder = Enc::new(template, end)?;
        for i in &self.data[0..begin] {
            encoder.push_ignore(&i.data_id);
        }
        encoder.init();
        let cache: *mut Cache<'a> = &mut self.cache;
        let encoder_ptr: *mut Enc = &mut encoder;
        Ok(join_all(self.data[begin..end].iter().zip(begin..end).map(
            async move |(data, index)| {
                let ret = if data.input.is_none() {
                    unsafe { &mut *cache }
                        .submit_iter(
                            (0..data.output_size).step_by(BLOCK).map(|x| SubmitKey {
                                test: index + 1,
                                time: x,
                            }),
                            template.language.as_str(),
                            |k| unsafe { &mut *encoder_ptr }.generate(k.time),
                        )
                        .await
                } else {
                    Vec::new()
                };
                unsafe { &mut *encoder_ptr }.push_ignore(&data.data_id);
                ret
            },
        ))
        .await)
    }
    async fn decode<Dec: DataDecoder, Err: ErrType>(
        &mut self,
        begin: usize,
        handles: Vec<Vec<Handle<Err>>>,
    ) -> Vec<Result<String, Error<Err, Dec::Error>>> {
        let mut decoder = Dec::new();
        let cache: *mut Cache<'_> = &mut self.cache;
        let data_ptr = &self.data;
        let decoder_ptr: *mut Dec = &mut decoder;
        join_all(
            handles
                .into_iter()
                .enumerate()
                .map(async move |(i, handle)| {
                    let data = &*data_ptr;
                    if let Some(p) = &data[begin + i].input {
                        Ok(p.clone())
                    } else {
                        let decoder = unsafe { &mut *decoder_ptr };
                        decoder.init(&data[begin + i]);
                        let ret = try {
                            unsafe { &mut *cache }
                                .get_result(handle)
                                .await
                                .into_iter()
                                .try_for_each(|v| {
                                    decoder.append_message(
                                        v.map_err(|e| Error::Submit(e))?.output.trim(),
                                    );
                                    Ok(())
                                })?;
                            decoder.decode().map_err(|e| Error::Decode(begin + i, e))?
                        };
                        decoder.clear();
                        ret
                    }
                }),
        )
        .await
    }
    pub async fn get_data<'c, 'b, Enc, Dec, Err>(
        &'c mut self,
        template: &Template,
        begin: usize,
        end: usize,
    ) -> DataResult<Err, Dec::Error>
    where
        Enc: DataEncoder<'b, Err>,
        Dec: DataDecoder,
        Err: ErrType,
        'a: 'c,
        'c: 'b,
    {
        let self_ptr: *mut Self = self;
        match unsafe { &mut *self_ptr }
            .fetch::<Enc, Err>(template, begin, end)
            .await
        {
            Ok(v) => DataResult::Result(self.decode::<Dec, Err>(begin, v).await),
            Err(e) => DataResult::Build(Error::Build(e)),
        }
    }
}
