extern crate serde;

use crate::{
    client::{Problem, Session, Verdict},
    encoding::{
        traits::{DataDecoder, DataEncoder, MetaEncoding},
        Template,
    },
    types::{Result, TestMeta, BLOCK},
};
use futures::executor::block_on;
use std::{fs::File, path, vec::Vec};

pub struct Downloader<'a> {
    session: &'a Session,
    problem: &'a Problem,
    pub testdata: Vec<TestMeta>,
}

impl<'a> Downloader<'a> {
    pub fn new(session: &'a Session, problem: &'a Problem) -> Self {
        Downloader {
            session,
            problem,
            testdata: Vec::new(),
        }
    }
    async fn submit_code(&self, lang: &'a str, code: &'a str) -> Result<Verdict> {
        self.session.submit(&self.problem, lang, code).await?;
        let sub = self.session.get_last_submission(&self.problem).await?;
        while !sub.is_judged(self.session).await {}
        sub.get_verdict(self.session).await
    }

    pub fn get_meta<'b, Enc>(&mut self, template: &Template, count: usize) -> Result<()>
    where
        Enc: MetaEncoding<'b>,
    {
        self.testdata.reserve(count);
        let base = self.testdata.len();
        let mut enc = Enc::new(template, count + base)?;
        unsafe {
            for i in 0..base {
                enc.ignore(&(*self.testdata.as_ptr().add(i)).data_id);
            }
        }
        for i in 0..count - 1 {
            self.testdata.push(Enc::decode(block_on(async {
                self.submit_code(template.language, enc.generate()?).await
            })?)?);
            unsafe {
                enc.ignore(&(*self.testdata.as_ptr().add(base + i)).data_id);
            }
        }
        Ok(())
    }
    pub fn load_meta(&mut self, dest: &path::Path) -> Result<()> {
        Ok(self.testdata = serde_yaml::from_reader(File::open(dest)?)?)
    }
    pub fn save_meta(&self, dest: &path::Path) -> Result<()> {
        Ok(serde_yaml::to_writer(File::create(dest)?, &self.testdata)?)
    }

    pub async fn get_data<'b, Enc, Dec>(
        &'b self,
        template: &Template<'b>,
        begin: usize,
        end: usize,
    ) -> Result<Vec<String>>
    where
        Enc: DataEncoder<'b>,
        Dec: DataDecoder,
    {
        let mut encoder = Enc::new(template, end)?;
        let mut decoder = Dec::new();
        for i in &self.testdata[0..begin] {
            encoder.push_ignore(&i.data_id);
        }
        let mut ret: Vec<String> = Vec::new();
        ret.reserve(end - begin);
        for i in &self.testdata[begin..end] {
            decoder.init(i);
            let count = (i.output_size + BLOCK - 1) / BLOCK;
            for j in 0..count {
                decoder.add_message(
                    j * BLOCK,
                    &self
                        .submit_code(template.language, encoder.generate(j * BLOCK)?)
                        .await?
                        .output,
                );
            }
            ret.push(decoder.decode()?);
            encoder.push_ignore(&i.data_id);
        }
        Ok(ret)
    }
}
