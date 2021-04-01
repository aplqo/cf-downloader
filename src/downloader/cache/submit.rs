extern crate futures;

use super::{Cache, SubmitKey};
use crate::{
    error::Error as ErrType,
    judge::{self, submit::Submission, Verdict},
    submitter,
};
use futures::future::join_all;
use std::{
    collections::HashMap, error::Error as StdError, fmt, iter::IntoIterator, mem::MaybeUninit,
    result::Result as StdResult,
};

#[derive(Debug)]
enum Kind<E: 'static + ErrType> {
    Submit(submitter::Error),
    GetResult(judge::Error),
    Generate(E),
}
#[derive(Debug)]
pub struct Error<E: 'static + ErrType> {
    kind: Kind<E>,
    id: SubmitKey,
}
impl<E: 'static + ErrType> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Submit(err) => write!(f, "Error submiting {}: {}", self.id, err),
            Kind::GetResult(err) => write!(f, "Error getting result for {}: {}", self.id, err),
            Kind::Generate(err) => write!(f, "Error generate code for {}: {}", self.id, err),
        }
    }
}
impl<E: 'static + ErrType> StdError for Error<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
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

enum State<E: 'static + ErrType> {
    Hit,
    Miss(Submission),
    Error(Kind<E>),
}
pub struct Handle<E: 'static + ErrType> {
    id: SubmitKey,
    state: State<E>,
}

impl<'a> Cache<'a> {
    pub async fn submit<Fun, Err>(
        &'a mut self,
        id: SubmitKey,
        language: &str,
        generate: Fun,
    ) -> StdResult<&'a Verdict, Error<Err>>
    where
        Fun: Fn(SubmitKey) -> StdResult<String, Err>,
        Err: 'static + ErrType,
    {
        if !self.cache.contains_key(&id) {
            self.cache.insert(
                id,
                self.submitter
                    .submit(
                        &self.problem,
                        language,
                        generate(id)
                            .map_err(|e| Error::new(id, Kind::Generate(e)))?
                            .as_str(),
                    )
                    .await
                    .map_err(|e| Error::new(id, Kind::Submit(e)))?
                    .wait(id.test)
                    .await
                    .map_err(|e| Error::new(id, Kind::GetResult(e)))?,
            );
        }
        Ok(self.cache.get(&id).unwrap())
    }

    pub async fn submit_iter<Fun, Iter, Err>(
        &mut self,
        iter: Iter,
        language: &str,
        generate: Fun,
    ) -> Vec<Handle<Err>>
    where
        Fun: Fn(SubmitKey) -> StdResult<String, Err>,
        Iter: IntoIterator<Item = SubmitKey>,
        Err: ErrType + 'static,
    {
        let mut ret: Vec<Handle<Err>> = Vec::new();
        let mut submit = Vec::new();
        let cache = &self.cache;
        self.submitter
            .submit_iter(
                &self.problem,
                language,
                iter.into_iter().enumerate().filter_map(|(index, id)| {
                    ret.push(Handle {
                        id,
                        state: unsafe { MaybeUninit::uninit().assume_init() },
                    });
                    match cache.get(&id) {
                        Some(_) => {
                            ret[index].state = State::Hit;
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
        ret
    }
    pub async fn get_result<Err: ErrType + 'static>(
        &'a mut self,
        mut handles: Vec<Handle<Err>>,
    ) -> Vec<StdResult<&'a Verdict, Error<Err>>> {
        {
            let cache: *mut HashMap<SubmitKey, Verdict> = &mut self.cache;
            unsafe {
                join_all(handles.iter_mut().map(async move |x| {
                    if let State::Miss(s) = &x.state {
                        match s.wait(x.id.time).await {
                            Ok(v) => {
                                (*cache).insert(x.id, v);
                            }
                            Err(e) => x.state = State::Error(Kind::GetResult(e)),
                        }
                    };
                }))
            }
            .await;
        }
        let cache = &self.cache;
        handles
            .into_iter()
            .map(|x| match x.state {
                State::Hit => Ok(cache.get(&x.id).unwrap()),
                State::Error(e) => Err(Error::new(x.id, e)),
                State::Miss(_) => Ok(cache.get(&x.id).unwrap()),
            })
            .collect()
    }
}
