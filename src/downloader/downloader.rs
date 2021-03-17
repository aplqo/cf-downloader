extern crate serde;

use crate::{
    client::{Problem, Session, Submission, Verdict},
    encoding::{
        traits::{DataDecoder, DataEncoder, MetaEncoding},
        Template,
    },
    types::{Error, Result, TestMeta, BLOCK},
};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs::File, path, time::Duration, vec::Vec};
use tokio::time::{sleep, sleep_until, Instant};
const UPDATE_RATE: usize = 7;
include!("./delay.rs");

#[derive(Serialize, Deserialize)]
struct DataList {
    problem: Problem,
    data: Vec<TestMeta>,
}
pub struct Downloader<'a> {
    session: &'a Session,
    list: DataList,
}

pub trait Callback {
    fn on_case_begin(&mut self, id: usize);
    fn on_case_end(&mut self, id: usize);
    fn on_progress(&mut self, _id: usize, _current: usize, _total: usize) {}
}

impl Submission {
    async fn wait_judge<'a>(self, session: &'a Session, id: usize) -> Result<Verdict> {
        let mut next = Instant::now();
        while !self.is_judged(session).await {
            next += CHECK_DELAY;
            sleep_until(next).await;
        }
        self.get_verdict(session, id).await
    }
}

impl<'a> Downloader<'a> {
    pub fn new(session: &'a Session, problem: Problem) -> Self {
        Downloader {
            session,
            list: DataList {
                problem,
                data: Vec::new(),
            },
        }
    }
    async fn submit_code(&self, lang: &String, code: &String) -> Result<Submission> {
        self.session
            .submit(&self.list.problem, lang.as_str(), code.as_str())
            .await?;
        sleep(SUBMISSION_GET_DELAY).await;
        self.session.get_last_submission(&self.list.problem).await
    }

    pub async fn get_meta<'b, Enc, F>(
        &mut self,
        template: &Template,
        count: usize,
        mut call: F,
    ) -> Result<()>
    where
        Enc: MetaEncoding<'b>,
        F: Callback,
    {
        self.list.data.reserve(count);
        let base = self.list.data.len();
        let mut enc = Enc::new(template, count + base)?;
        unsafe {
            for i in 0..base {
                enc.ignore(&(*self.list.data.as_ptr().add(i)).data_id);
            }
        }
        let mut next = Instant::now();
        for i in 0..count {
            sleep_until(next).await;
            call.on_case_begin(i + base);
            self.list.data.push(Enc::decode(
                self.submit_code(&template.language, enc.generate()?)
                    .await?
                    .wait_judge(&self.session, base + i + 1)
                    .await?,
            )?);
            next += SUBMIT_DELAY;
            unsafe {
                enc.ignore(&(*self.list.data.as_ptr().add(base + i)).data_id);
            }
            call.on_case_end(i + base);
        }
        Ok(())
    }
    pub fn load_meta(&mut self, dest: &path::Path) -> Result<()> {
        let lst: DataList = serde_yaml::from_reader(File::open(dest)?)?;
        if lst.problem != self.list.problem {
            return Err(Error::new(format!(
                "Problem mismatch. Selected {:#?}. But loading {:#?}",
                self.list.problem, lst.problem
            )));
        }
        Ok(self.list = lst)
    }
    pub fn save_meta(&self, dest: &path::Path) -> Result<()> {
        Ok(serde_yaml::to_writer(File::create(dest)?, &self.list)?)
    }
    pub async fn get_data<'b, Enc, Dec, F>(
        &'b self,
        template: &Template,
        begin: usize,
        end: usize,
        mut call: F,
    ) -> Result<Vec<String>>
    where
        Enc: DataEncoder<'b>,
        Dec: DataDecoder,
        F: Callback,
    {
        let length = end - begin;
        let mut ret: Vec<String> = Vec::new();
        let mut verdicts = VecDeque::new();
        verdicts.reserve(length);
        {
            let mut encoder = Enc::new(template, end)?;
            let mut next = Instant::now();
            for i in &self.list.data[0..begin] {
                encoder.push_ignore(&i.data_id);
            }
            for (ind, i) in self.list.data[begin..end].into_iter().enumerate() {
                let mut cur = VecDeque::new();
                if let None = &i.input {
                    call.on_case_begin(ind + begin);
                    let count = (i.output_size + BLOCK - 1) / BLOCK;
                    cur.reserve(count);
                    for j in 0..count {
                        cur.push_back(
                            self.submit_code(&template.language, encoder.generate(j * BLOCK)?)
                                .await?
                                .wait_judge(&self.session, ind + begin + 1),
                        );
                        next += SUBMIT_DELAY;
                        if j & UPDATE_RATE == 0 {
                            call.on_progress(ind + begin, j, count);
                        }
                    }
                }
                encoder.push_ignore(&i.data_id);
                verdicts.push_back(cur);
            }
        }
        ret.reserve(length);
        {
            let mut decoder = Dec::new();
            for i in 0..length {
                call.on_case_end(i + begin);
                if let Some(p) = &self.list.data[begin + i].input {
                    ret.push(p.clone());
                } else {
                    decoder.init(&self.list.data[begin + i]);
                    let mut offset: usize = 0;
                    for s in try_join_all(verdicts.pop_front().unwrap()).await? {
                        decoder.add_message(offset, s.output.trim());
                        offset += BLOCK;
                    }
                    ret.push(decoder.decode()?);
                    decoder.clear();
                }
            }
        }
        Ok(ret)
    }
    pub fn len(&self) -> usize {
        self.list.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.list.data.is_empty()
    }
}
