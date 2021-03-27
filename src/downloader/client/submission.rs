extern crate reqwest;
extern crate tokio;

use super::{retry::async_retry, verdict::Verdict};
use crate::{
    config::submission::CHECK_DELAY,
    types::{Error, Result},
};
use reqwest::Client;
use std::collections::HashMap;
use tokio::time::{sleep_until, Instant};

const MAX_OUTPUT: usize = 500;

pub struct Submission {
    pub(super) client: Client,
    pub(super) id: String,
    pub(super) csrf_token: String,
}
impl Submission {
    fn full_data_or(data: String) -> Option<String> {
        if data.len() > MAX_OUTPUT {
            None
        } else {
            Some(data)
        }
    }
    pub async fn poll(&self, id: usize) -> Result<Option<Verdict>> {
        let mut data = async_retry(async || {
            self.client
                .post("https://codeforces.com/data/submitSource")
                .form(&[("submissionId", &self.id), ("csrf_token", &self.csrf_token)])
                .send()
                .await?
                .error_for_status()?
                .json::<HashMap<String, String>>()
                .await
        })
        .await?;
        if data["verdict"].contains("verdict-waiting") {
            return Ok(None);
        } else {
            let pos = data.remove("testCount").unwrap();
            if pos.parse::<usize>().unwrap() != id {
                return Err(Error::new(String::from("Test count not expected")));
            }
            return Ok(Some(Verdict {
                input: Self::full_data_or(data.remove(&format!("input#{}", pos)).unwrap()),
                output: data.remove(&format!("output#{}", pos)).unwrap(),
                answer: Self::full_data_or(data.remove(&format!("answer#{}", pos)).unwrap()),
            }));
        }
    }
    pub async fn wait(&self, id: usize) -> Result<Verdict> {
        let mut next = Instant::now();
        loop {
            sleep_until(next).await;
            if let Some(v) = self.poll(id).await? {
                return Ok(v);
            }
            next += CHECK_DELAY;
        }
    }
}
