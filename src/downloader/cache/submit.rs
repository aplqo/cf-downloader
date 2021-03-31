extern crate futures;

use super::{Cache, SubmitKey};
use crate::{
    error::Error as ErrType,
    judge::{self, submit::Submission, Verdict},
    submitter,
};
use futures::future::join_all;
use std::{
    error::Error as StdError, fmt, iter::IntoIterator, mem::MaybeUninit,
    result::Result as StdResult,
};

#[derive(Debug)]
enum Kind<E: ErrType> {
    Submit(submitter::Error),
    GetResult(judge::Error),
    Generate(E),
}
#[derive(Debug)]
pub struct Error<E: ErrType> {
    kind: Kind<E>,
    id: SubmitKey,
}
impl<E: ErrType> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Kind::Submit(err) => write!(f, "Error submiting {}: {}", self.id, err),
            Kind::GetResult(err) => write!(f, "Error getting result for {}: {}", self.id, err),
            Kind::Generate(err) => write!(f, "Error generate code for {}: {}", self.id, err),
        }
    }
}
impl<E: ErrType> StdError for Error<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Kind::Submit(err) => Some(err),
            Kind::GetResult(err) => Some(err),
            Kind::Generate(err) => Some(err),
        }
    }
}
impl<E: ErrType> Error<E> {
    fn new(id: SubmitKey, kind: Kind<E>) -> Self {
        Self { id, kind }
    }
}

enum State<'a, E: ErrType> {
    Hit(&'a Verdict),
    Miss(Submission),
    Error(Kind<E>),
}
pub struct Handle<'a, E: ErrType> {
    id: SubmitKey,
    state: State<'a, E>,
}

impl<'a, Err: ErrType> Cache<'a> {
    pub async fn submit<Fun>(
        &mut self,
        id: SubmitKey,
        language: &str,
        generate: Fun,
    ) -> StdResult<Verdict, Error<Err>>
    where
        Fun: Fn(SubmitKey) -> StdResult<String, Err>,
    {
        match self.cache.get(id) {
            Some(v) => Ok(v),
            None => {
                self.cache.insert(
                    id,
                    self.submitter
                        .submit(
                            self.problem,
                            language,
                            generate(id).map_err(|e| Error::new(id, Kind::Generate(e)))?,
                        )
                        .await
                        .map_err(|e| Error::new(id, Kind::Submit(e)))?
                        .wait(id.offset)
                        .await
                        .map_err(|e| Error::new(id, Kind::GetResult(e)))?,
                );
                Ok(self.cache.get(id).unwrap())
            }
        }
    }
    pub async fn submit_iter<Fun, Iter>(
        &self,
        iter: Iter,
        language: &str,
        generate: Fun,
    ) -> Vec<Handle<'_, Err>>
    where
        Fun: Fn(SubmitKey) -> StdResult<String, Err>,
        Iter: IntoIterator<Item = SubmitKey>,
    {
        let mut ret: Vec<Handle<'_, Err>> = vec![unsafe { MaybeUninit::uninit() }; iter.len()];
        let mut submit = Vec::new();
        self.submitter
            .submit_iter(
                self.problem,
                language,
                iter.into_iter().enumerate().filter_map(|(index, id)| {
                    ret[index].id = id;
                    match self.cache.get(id) {
                        Some(v) => {
                            ret[index].state = State::Hit(v);
                            None
                        }
                        None => match generate(id) {
                            Ok(v) => {
                                submit.push(index);
                                Some(v)
                            }
                            Err(e) => {
                                ret[index].state = State::Error(Kind::Generate(e));
                                None
                            }
                        },
                    }
                }),
            )
            .await
            .into_iter()
            .zip(submit)
            .map(|(v, id)| match v {
                Ok(s) => ret[id].state = State::Miss(s),
                Err(e) => ret[id].state = State::Error(Kind::Submit(e)),
            });
        return ret;
    }
    pub async fn get_result(
        &mut self,
        handles: Vec<Handle<'_, Err>>,
    ) -> Vec<StdResult<&Verdict, Error<Err>>> {
        join_all(handles.into_iter().map(async move |x| match x.state {
            State::Hit(v) => Ok(v),
            State::Miss(s) => match s.wait(x.id.time).await {
                Ok(v) => {
                    self.cache.insert(x.id, v);
                    Ok(self.cache.get(x.id).unwrap())
                }
                Err(e) => Err(Error::new(x.id, Kind::GetResult(e))),
            },
            State::Error(e) => Err(Error::new(x.id, e)),
        }))
        .await
    }
}
