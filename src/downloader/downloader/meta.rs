use super::Downloader;

use crate::{
    encoding::{MetaEncoding, Template},
    error::Error as ErrType,
    judge, submitter,
};
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
enum Kind<E: ErrType + 'static> {
    Build(E),
    Submit(submitter::Error),
    Generate(E),
    GetResult(judge::Error),
    Decode(E),
}
#[derive(Debug)]
pub struct Error<E: ErrType + 'static> {
    id: usize,
    kind: Kind<E>,
}
impl<E: ErrType + 'static> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Build(e) => write!(f, "Error building template {}", e),
            Kind::Generate(e) => write!(f, "Error generating code for {}: {}", self.id, e),
            Kind::Submit(e) => write!(f, "Error submit code for {}: {}", self.id, e),
            Kind::GetResult(e) => write!(f, "Error getting result for {}: {}", self.id, e),
            Kind::Decode(e) => write!(f, "Error decoding result for {}: {}", self.id, e),
        }
    }
}
impl<E: ErrType + 'static> StdError for Error<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            Kind::Build(e) => Some(e),
            Kind::Generate(e) => Some(e),
            Kind::Submit(e) => Some(e),
            Kind::GetResult(e) => Some(e),
            Kind::Decode(e) => Some(e),
        }
    }
}
impl<E: ErrType> Error<E> {
    fn new(id: usize, kind: Kind<E>) -> Self {
        Self { id, kind }
    }
    fn from_build(error: E) -> Self {
        Self {
            id: 0,
            kind: Kind::Build(error),
        }
    }
}

impl<'a> Downloader<'a> {
    pub async fn get_meta<Enc>(
        &mut self,
        template: &Template,
        end: usize,
    ) -> Result<(), Error<<Enc as MetaEncoding<'a>>::Error>>
    where
        Enc: MetaEncoding<'a> + 'static,
    {
        if end < self.len() {
            return Ok(());
        }
        let base = self.data.len();
        let count = end - base;
        self.data.reserve(count);
        let mut enc = Enc::new(template, count + base).map_err(Error::from_build)?;
        unsafe {
            for i in 0..base {
                enc.ignore(&(*self.data.as_ptr().add(i)).data_id);
            }
        }
        enc.init();
        for id in base..base + count {
            self.data.push(
                Enc::decode(
                    self.cache
                        .submitter
                        .submit(
                            &self.problem,
                            &template.language,
                            enc.generate()
                                .map_err(|e| Error::new(id, Kind::Generate(e)))?
                                .as_str(),
                        )
                        .await
                        .map_err(|e| Error::new(id, Kind::Submit(e)))?
                        .wait(id)
                        .await
                        .map_err(|e| Error::new(id, Kind::GetResult(e)))?,
                )
                .map_err(|e| Error::new(id, Kind::Decode(e)))?,
            );
            unsafe {
                enc.ignore(&(*self.data.as_ptr().add(id)).data_id);
            }
        }
        Ok(())
    }
}
