extern crate regex;
extern crate reqwest;
extern crate tokio;

use super::{
    error::{Error, Kind, Result},
    problem::Problem,
    retry::async_retry,
    search::search_response,
    Session, Verdict,
};
use crate::config::judge::{session::BFAA, submit::CHECK_DELAY};
use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;
use tokio::time::{sleep_until, Instant};

const MAX_OUTPUT: usize = 500;

pub(super) struct RegexSet {
    submit: Regex,
    last_submit: Regex,
}
impl RegexSet {
    pub(super) fn new() -> Self {
        Self {
            submit: Regex::new(r#"error[a-zA-Z_\-\\ ]*">(.*)</span>"#).unwrap(),
            last_submit: Regex::new(r#"data-submission-id="([[:digit:]]+)""#).unwrap(),
        }
    }
}

pub struct Submission {
    client: Client,
    id: String,
    csrf_token: String,
}
fn full_data_or(data: String) -> Option<String> {
    if data.len() > MAX_OUTPUT {
        None
    } else {
        Some(data)
    }
}
impl Submission {
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
            let pos = data.remove("testCount").unwrap().parse::<usize>();
            if pos != id {
                return Err(Error::with_kind(Kind::TestCount(pos, id)));
            }
            return Ok(Some(Verdict {
                input: full_data_or(data.remove(&format!("input#{}", pos)).unwrap()),
                output: data.remove(&format!("output#{}", pos)).unwrap(),
                answer: full_data_or(data.remove(&format!("answer#{}", pos)).unwrap()),
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

impl Session {
    pub async fn get_last_submission(&self, problem: &Problem) -> Result<Submission> {
        let csrf = self.get_csrf(problem.status_url.as_str()).await?;
        Ok(Submission {
            id: search_response(
                || {
                    self.client
                        .post(&problem.status_url)
                        .query(&[("order", "BY_ARRIVED_DESC")])
                        .form(&[
                            ("csrf_token", csrf.as_str()),
                            ("action", "setupSubmissionFilter"),
                            ("frameProblemIndex", problem.id.as_str()),
                            ("verdictName", "anyVerdict"),
                            ("programTypeForInvoker", "anyProgramTypeForInvoker"),
                            ("comparisonType", "NOT_USED"),
                            ("judgedTestCount", ""),
                            ("participantSubstring", self.handle.as_str()),
                            ("_tta", "54"),
                        ])
                },
                &self.regex.submit.last_submit,
            )
            .await?
            .ok_or_else(|| Error::with_kind(Kind::Regex))?,
            client: self.client.clone(),
            csrf_token: csrf,
        })
    }
    pub async fn submit(&self, problem: &Problem, language: &str, code: &str) -> Result<()> {
        let csrf = self.get_csrf(problem.submit_url.as_str()).await?;
        search_response(
            || {
                self.client
                    .post(&problem.submit_url)
                    .query(&[("csrf_token", csrf.as_str())])
                    .form(&[
                        ("csrf_token", csrf.as_str()),
                        ("ftaa", self.ftaa.as_str()),
                        ("bfaa", BFAA),
                        ("action", "submitSolutionFormSubmitted"),
                        ("submittedProblemIndex", problem.id.as_str()),
                        ("programTypeId", language),
                        ("contestId", problem.contest.as_str()),
                        ("source", code),
                        ("tabSize", "4"),
                        ("_tta", "594"),
                        ("sourceCodeConfirmed", "true"),
                    ])
            },
            &self.regex.submit.submit,
        )
        .await?
        .map_or(Ok(()), |x| Err(Error::with_description(Kind::API, x)))
    }
}
