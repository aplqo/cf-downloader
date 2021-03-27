extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde;

use super::{
    problem::{get_problem_url, Problem, Type},
    retry::async_retry,
    submission::Submission,
};
use crate::types::{Error, Result};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;
use reqwest::{Client, Proxy, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::iter;

const BFAA: &str = "f1b3f18c715565b589b7823cda7448ce";
const FIREFOX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

struct UtilityRegex {
    csrf: Regex,
    login: Regex,
    submit: Regex,
    last_submit: Regex,
    logout: Regex,
}
impl UtilityRegex {
    fn new() -> Self {
        UtilityRegex {
            csrf: Regex::new("csrf='(.+)'").unwrap(),
            login: Regex::new(r#"handle = "[[:word:]]+"#).unwrap(),
            submit: Regex::new(r#"error[a-zA-Z_\-\\ ]*">(.*)</span>"#).unwrap(),
            last_submit: Regex::new(r#"data-submission-id="([[:digit:]]+)""#).unwrap(),
            logout: Regex::new(r#"<a href="/([[:xdigit:]]+)/logout""#).unwrap(),
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

fn search_text(str: String, regex: &Regex, error: &str) -> Result<String> {
    match regex.captures(str.as_str()) {
        Some(v) => Ok(v.get(1).unwrap().as_str().to_owned()),
        None => Err(Error::new(error.to_string())),
    }
}
async fn search_response<T: Fn() -> RequestBuilder>(
    fun: T,
    regex: &Regex,
    error: &str,
) -> Result<String> {
    search_text(
        async_retry(async || fun().send().await?.error_for_status()?.text().await).await?,
        regex,
        error,
    )
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub handle: String,
    pub password: String,
    pub proxy: Option<String>,
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
    pub async fn from_login(login: Account) -> Result<Self> {
        let ret = Self::new(login.handle, login.proxy)?;
        ret.login(login.password.as_str()).await?;
        Ok(ret)
    }

    async fn get_csrf(&self, url: &str) -> Result<String> {
        search_response(
            || self.client.get(url),
            &self.regex.csrf,
            "Regex to find csrf token not matched",
        )
        .await
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
                .error_for_status()?
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
                &self.regex.last_submit,
                "Can't find last submission url",
            )
            .await?,
            client: self.client.clone(),
            csrf_token: csrf,
        })
    }
    pub async fn submit(&self, problem: &Problem, language: &str, code: &str) -> Result<()> {
        let csrf = self.get_csrf(problem.submit_url.as_str()).await?;
        match self.regex.submit.captures(
            async_retry(async || {
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
                    .error_for_status()?
                    .text()
                    .await
            })
            .await?
            .as_str(),
        ) {
            Some(err) => Err(Error::new(err.get(1).unwrap().as_str().to_string())),
            None => Ok(()),
        }
    }
    pub async fn check_exist(&self, source: Type, contest: &str, id: &str) -> Result<bool> {
        let url = get_problem_url(source, contest, id);
        Ok(url
            == self
                .client
                .get(&url)
                .send()
                .await?
                .error_for_status()?
                .url()
                .as_str())
    }
    pub async fn logout(&self) -> Result<()> {
        let url = search_response(
            || self.client.get("https://codeforces.com"),
            &self.regex.logout,
            "Logout url regex mismatch",
        )
        .await?;
        async_retry(async || {
            self.client
                .get(format!("https://codeforces.com/{}/logout", url))
                .send()
                .await?
                .error_for_status()
        })
        .await?;
        Ok(())
    }
}
