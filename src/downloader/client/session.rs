extern crate rand;
extern crate regex;
extern crate reqwest;

use super::{
    problem::{get_problem_url, Problem, Type},
    retry::async_retry,
    submission::Submission,
};
use crate::types::{Error, Result};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;
use reqwest::{Client, Proxy};
use std::iter;

const BFAA: &str = "f1b3f18c715565b589b7823cda7448ce";
const FIREFOX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

struct UtilityRegex {
    csrf: Regex,
    login: Regex,
    submit: Regex,
    last_submit: Regex,
}
impl UtilityRegex {
    fn new() -> Self {
        UtilityRegex {
            csrf: Regex::new("csrf='(.+)'").unwrap(),
            login: Regex::new(r#"handle = "[[:word:]]+"#).unwrap(),
            submit: Regex::new(r#"error[a-zA-Z_\-\\ ]*">(.*)</span>"#).unwrap(),
            last_submit: Regex::new(r#"data-submission-id="([[:digit:]]+)""#).unwrap(),
        }
    }
}

fn random_string() -> String {
    iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(18)
        .collect()
}

pub struct Session {
    pub(super) client: Client,
    handle: String,
    ftaa: String,
    regex: UtilityRegex,
}
impl Session {
    pub fn new(handle: String, proxy: Option<String>) -> Result<Self> {
        let mut builder = Client::builder();
        if let Some(p) = proxy {
            builder = builder.proxy(Proxy::https(p)?);
        }
        Ok(Session {
            client: builder
                .user_agent(FIREFOX_UA)
                .cookie_store(true)
                .build()
                .unwrap(),
            handle,
            ftaa: random_string(),
            regex: UtilityRegex::new(),
        })
    }

    async fn get_csrf(&self, url: &str) -> Result<String> {
        let body = async_retry(async || self.client.get(url).send().await?.text().await).await?;
        Ok(self
            .regex
            .csrf
            .captures(body.as_str())
            .expect("Regex to find csrf token not matched")
            .get(1)
            .unwrap()
            .as_str()
            .to_string())
    }

    pub async fn login(&self, password: &str) -> Result<()> {
        const URL: &str = "https://codeforces.com/enter";
        let csrf = self.get_csrf(URL).await?;
        let body = async_retry(async || {
            self.client
                .post(URL)
                .form(&[
                    ("csrf_token", csrf.as_str()),
                    ("action", "enter"),
                    ("ftaa", self.ftaa.as_str()),
                    ("bfaa", BFAA),
                    ("handleOrEmail", self.handle.as_str()),
                    ("password", password),
                    ("_tta", "176"),
                    ("remember", "off"),
                ])
                .send()
                .await?
                .text()
                .await
        })
        .await?;
        if self.regex.login.is_match(body.as_str()) {
            Ok(())
        } else {
            Err(Error::new("Failed to log into codeforces.com".to_string()))
        }
    }
    pub async fn get_last_submission(&self, problem: &Problem) -> Result<Submission<'_>> {
        let csrf = self.get_csrf(problem.status_url.as_str()).await?;
        let body = async_retry(async || {
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
                .send()
                .await?
                .text()
                .await
        })
        .await?;
        match self.regex.last_submit.captures(body.as_str()) {
            Some(id) => Ok(Submission {
                session: self,
                id: id.get(1).unwrap().as_str().to_string(),
                csrf_token: csrf,
            }),
            None => Err(Error::new("Can't find last submission url".to_string())),
        }
    }
    pub async fn submit(&self, problem: &Problem, language: &str, code: &str) -> Result<()> {
        let csrf = self.get_csrf(problem.submit_url.as_str()).await?;
        let body = async_retry(async || {
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
                .send()
                .await?
                .text()
                .await
        })
        .await?;
        match self.regex.submit.captures(body.as_str()) {
            Some(err) => Err(Error::new(err.get(1).unwrap().as_str().to_string())),
            None => Ok(()),
        }
    }
    pub async fn check_exist(&self, source: Type, contest: &str, id: &str) -> Result<bool> {
        let url = get_problem_url(source, contest, id);
        Ok(url == self.client.get(&url).send().await?.url().as_str())
    }
    pub async fn logout(&self) -> Result<()> {
        self.client
            .get("https://codeforces.com/logout")
            .send()
            .await?;
        Ok(())
    }
}
