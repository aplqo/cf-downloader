extern crate reqwest;

use super::{retry::async_retry, session::Session, verdict::Verdict};
use crate::types::{Error, Result};
use std::collections::HashMap;

const MAX_OUTPUT: usize = 500;

pub struct Submission<'a> {
    pub(super) session: &'a Session,
    pub(super) id: String,
    pub(super) csrf_token: String,
}
impl<'a> Submission<'a> {
    fn full_data_or(data: String) -> Option<String> {
        if data.len() > MAX_OUTPUT {
            None
        } else {
            Some(data)
        }
    }
    pub async fn poll(&self, id: usize) -> Result<Option<Verdict>> {
        let mut data = async_retry(async || {
            self.session
                .client
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
}
