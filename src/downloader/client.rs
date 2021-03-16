extern crate regex;
extern crate reqwest;
extern crate serde;

use crate::types::{Error, Result};
use regex::Regex;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, result::Result as StdResult};

const MAX_OUTPUT: usize = 500;
const BFAA: &str = "f1b3f18c715565b589b7823cda7448ce";
const FIREFOX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ProblemType {
    Contest,
    Gym,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Problem {
    pub source: ProblemType,
    pub contest: String,
    pub id: String,
}
pub struct Verdict {
    pub(crate) input: Option<String>,
    pub(crate) output: String,
    pub(crate) answer: Option<String>,
}

pub struct Submission {
    pub id: String,
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
    async fn get_info(
        &self,
        session: &Session,
    ) -> StdResult<HashMap<String, String>, reqwest::Error> {
        session
            .client
            .post("https://codeforces.com/data/submitSource")
            .form(&[("submissionId", &self.id), ("csrf_token", &self.csrf_token)])
            .send()
            .await?
            .json::<HashMap<String, String>>()
            .await
    }
    pub async fn is_judged(&self, session: &Session) -> bool {
        self.get_info(session).await.map_or(false, |v| {
            let p = &v["verdict"];
            !p.contains("verdict-waiting")
        })
    }
    pub async fn get_verdict(&self, session: &Session, id: usize) -> Result<Verdict> {
        let mut data = self.get_info(session).await?;
        let pos = data.remove("testCount").unwrap();
        if pos.parse::<usize>().unwrap() != id {
            return Err(Error::new(String::from("Test count not expected")));
        }
        Ok(Verdict {
            input: full_data_or(data.remove(&format!("input#{}", pos)).unwrap()),
            output: data.remove(&format!("output#{}", pos)).unwrap(),
            answer: full_data_or(data.remove(&format!("answer#{}", pos)).unwrap()),
        })
    }
}
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
pub struct Session {
    client: Client,
    handle: String,
    ftaa: String,
    regex: UtilityRegex,
}
impl Session {
    pub fn new(handle: String, proxy: &str) -> Result<Self> {
        Ok(Session {
            client: Client::builder()
                .proxy(Proxy::https(proxy)?)
                .user_agent(FIREFOX_UA)
                .cookie_store(true)
                .build()
                .unwrap(),
            handle,
            ftaa: "123456789123456789".to_string(),
            regex: UtilityRegex::new(),
        })
    }

    async fn get_csrf(&self, url: &str) -> StdResult<String, reqwest::Error> {
        let body = self.client.get(url).send().await?.text().await?;
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
        let body = self
            .client
            .post(URL)
            .query(&[
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
            .await?;
        if self.regex.login.is_match(body.as_str()) {
            Ok(())
        } else {
            Err(Error::new("Failed to log into codeforces.com".to_string()))
        }
    }
    pub async fn get_last_submission(&self, problem: &Problem) -> Result<Submission> {
        let url = format!("https://codeforces.com/contest/{}/status", problem.contest);
        let csrf = self.get_csrf(url.as_str()).await?;
        let body = self
            .client
            .post(url)
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
            .await?;
        match self.regex.last_submit.captures(body.as_str()) {
            Some(id) => Ok(Submission {
                id: id.get(1).unwrap().as_str().to_string(),
                csrf_token: csrf,
            }),
            None => Err(Error::new("Can't find last submission url".to_string())),
        }
    }
    pub async fn submit(&self, problem: &Problem, language: &str, code: &str) -> Result<()> {
        let url = format!("https://codeforces.com/contest/{}/submit", problem.contest);
        let csrf = self.get_csrf(url.as_str()).await?;
        let body = self
            .client
            .post(format!("{}?csrf_token={}", url, csrf))
            .query(&[
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
            .await?;
        match self.regex.submit.captures(body.as_str()) {
            Some(err) => Err(Error::new(err.get(1).unwrap().as_str().to_string())),
            None => Ok(()),
        }
    }
    pub async fn check_exist(&self, problem: &Problem) -> Result<bool> {
        Ok(self
            .client
            .get(format!(
                "https://codeforces.com/contest/{}/problem/{}",
                problem.contest, problem.id
            ))
            .send()
            .await?
            .url()
            .as_str()
            != "https://codeforces.com")
    }

    pub async fn logout(&self) -> Result<()> {
        self.client
            .get("https://codeforces.com/logout")
            .send()
            .await?;
        Ok(())
    }
}
