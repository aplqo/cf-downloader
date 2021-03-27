extern crate serde;

use crate::{
    encoding::{
        traits::{DataDecoder, DataEncoder, MetaEncoding},
        Template,
    },
    judge::problem::Problem,
    submitter::Submitter,
    types::{Error, Result, TestMeta, BLOCK},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path, vec::Vec};

#[derive(Serialize, Deserialize)]
struct DataList {
    problem: Problem,
    data: Vec<TestMeta>,
}
pub struct Downloader {
    list: DataList,
}

pub trait Callback {
    fn on_case_begin(&mut self, id: usize);
    fn on_case_end(&mut self, id: usize);
    fn on_progress(&mut self, _id: usize, _current: usize, _total: usize) {}
}

impl Downloader {
    pub fn new(problem: Problem) -> Self {
        Downloader {
            list: DataList {
                problem,
                data: Vec::new(),
            },
        }
    }

    pub async fn get_meta<'b, Enc, F>(
        &mut self,
        submitter: &mut Submitter,
        template: &Template,
        end: usize,
        mut call: F,
    ) -> Result<()>
    where
        Enc: MetaEncoding<'b>,
        F: Callback,
    {
        if end < self.len() {
            return Ok(());
        }
        let base = self.list.data.len();
        let count = end - base;
        self.list.data.reserve(count);
        let mut enc = Enc::new(template, count + base)?;
        unsafe {
            for i in 0..base {
                enc.ignore(&(*self.list.data.as_ptr().add(i)).data_id);
            }
        }
        enc.init();
        for i in 0..count {
            call.on_case_begin(i + base);
            self.list.data.push(Enc::decode(
                submitter
                    .submit(
                        &self.list.problem,
                        &template.language,
                        enc.generate()?.as_str(),
                    )
                    .await?
                    .wait(base + i + 1)
                    .await?,
            )?);
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
        self.list = lst;
        Ok(())
    }
    pub fn save_meta(&self, dest: &path::Path) -> Result<()> {
        Ok(serde_yaml::to_writer(File::create(dest)?, &self.list)?)
    }
    pub async fn get_data<'b, Enc, Dec, F>(
        &'b self,
        submitter: &mut Submitter,
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
        let mut verdicts = Vec::with_capacity(length);
        {
            let mut encoder = Enc::new(template, end)?;
            for i in &self.list.data[0..begin] {
                encoder.push_ignore(&i.data_id);
            }
            encoder.init();
            for i in &self.list.data[begin..end] {
                if i.input.is_none() {
                    let count = (i.output_size + BLOCK - 1) / BLOCK;
                    let mut code = Vec::with_capacity(count);
                    for j in 0..count {
                        code.push(encoder.generate(j * BLOCK)?);
                    }
                    verdicts.push(
                        submitter
                            .submit_vec(&self.list.problem, template.language.as_str(), code)
                            .await?,
                    );
                } else {
                    verdicts.push(Vec::new());
                }
                encoder.push_ignore(&i.data_id);
            }
        }
        let mut ret: Vec<String> = Vec::new();
        ret.reserve(length);
        {
            let mut decoder = Dec::new();
            for (i, it) in verdicts.iter().enumerate() {
                call.on_case_end(i + begin);
                if let Some(p) = &self.list.data[begin + i].input {
                    ret.push(p.clone());
                } else {
                    decoder.init(&self.list.data[begin + i]);
                    for s in it {
                        decoder.append_message(s.wait(i + begin + 1).await?.output.trim());
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
