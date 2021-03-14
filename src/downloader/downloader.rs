extern crate serde;

use crate::{
    client::{Problem, Session},
    encoding::traits::{DataDecoder, DataEncoder, MetaEncoding},
    types::{Error, Template, TestMeta, BLOCK},
};
use futures::executor::block_on;
use std::{borrow::Borrow, thread::sleep, time::Duration, vec::Vec};

pub struct Downloader<'a> {
    session: &'a Session,
    delay: Duration,
    problem: &'a Problem,
    testdata: Vec<TestMeta>,
}

impl<'a> Downloader<'a> {
    pub fn new(session: &'a Session, delay: Duration, problem: &'a Problem) -> Self {
        Downloader {
            session,
            delay,
            problem,
            testdata: Vec::new(),
        }
    }
    pub fn get_meta<Enc, T>(
        &mut self,
        template: &Template,
        count: usize,
        call: T,
    ) -> Result<(), Error>
    where
        Enc: MetaEncoding,
        T: Fn(usize) -> (),
    {
        self.testdata.reserve(count);
        let mut enc = Enc::new(template, count);
        for i in 0..count - 1 {
            call(i);
            self.testdata.push(Enc::decode(block_on(async {
                self.session
                    .submit(self.problem, template.language, enc.generate_encoder())
                    .await?;
                let url = self.session.get_last_submission_url(self.problem).await?;
                self.session.wait_submission(&url).await
            })?)?);
            sleep(self.delay);
            enc.ignore_input(&self.testdata[i].hash);
        }
        Ok(())
    }
}
