extern crate regex;
extern crate reqwest;

use crate::types::{Error, Result};
use futures::executor::block_on;
use regex::Regex;
use reqwest::{Client, Proxy};
use std::{collections::HashMap, result::Result as StdResult};

const BFAA: &str = "f1b3f18c715565b589b7823cda7448ce";
const FIREFOX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

pub enum ProblemType {
    Contest,
    Gym,
}
pub struct Problem {
    pub source: ProblemType,
    pub contest: String,
    pub id: String,
}
pub struct Verdict {
    pub output: String,
    pub answer: Option<String>,
}

impl Problem {}

pub struct Submission {
    id: String,
    csrf_token: String,
}
impl Submission {
    async fn get_info(
        &self,
        session: &Session,
    ) -> StdResult<HashMap<String, String>, reqwest::Error> {
        session
            .client
            .get("https://codeforces.com/data/submitSource")
            .query(&[("submissionId", &self.id), ("csrf_token", &self.csrf_token)])
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
    pub async fn get_verdict(&self, session: &Session) -> Result<Verdict> {
        let mut data = self.get_info(session).await?;
        let pos = data.remove("testCount").unwrap();
        let output = data.remove(&format!("output#{}", pos)).unwrap();
        let answer = data.remove(&format!("answer#{}", pos)).unwrap();
        Ok(Verdict {
            output,
            answer: if answer.len() > 500 {
                None
            } else {
                Some(answer)
            },
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
            csrf: Regex::new("csrf='(.+?)'").unwrap(),
            login: Regex::new(r#"handle = "([\s\S]+?)"#).unwrap(),
            submit: Regex::new(r#"error[a-zA-Z_\-\ ]*">(.*?)</span>"#).unwrap(),
            last_submit: Regex::new(r#"<tr class="last-row" data-submission-id="([[:digit:]]+)""#)
                .unwrap(),
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

    pub fn login(&self, password: &str) -> Result<()> {
        const URL: &str = "https://codeforces.com/enter";
        let body = block_on(async {
            let csrf = self.get_csrf(URL).await?;
            self.client
                .post(URL)
                .query(&[
                    ("csrf_token", csrf.as_str()),
                    ("action", "enter"),
                    ("ftaa", self.ftaa.as_str()),
                    ("bfaa", BFAA),
                    ("handle_or_email", self.handle.as_str()),
                    ("password", password),
                    ("_tta", "176"),
                    ("remember", "off"),
                ])
                .send()
                .await?
                .text()
                .await
        })?;
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
            .query(&[
                ("csrf_token", csrf.as_str()),
                ("action", "setupSubmissionFilter"),
                ("frameProblemIndex", problem.id.as_str()),
                ("verdictName", "anyVerdict"),
                ("programTypeForInvoker", "anyProgramTypeForInvoker"),
                ("comparisonType", "NOT_USED"),
                ("judgedTestCount", ""),
                ("participantSubstring", self.handle.as_str()),
                ("_tta", "373"),
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
        let csrf = self.get_csrf("https://codeforces.com").await?;
        let body = self
            .client
            .post(format!(
                "https://codeforces.com/submit?csrf_token={}",
                csrf.as_str()
            ))
            .query(&[
                ("csrf_token", csrf.as_str()),
                ("ftaa", self.ftaa.as_str()),
                ("bfaa", BFAA),
                ("action", "submitSolutionFormSubmitted"),
                ("problemTypeId", language),
                ("submittedProblemIndex", problem.id.as_str()),
                ("contestId", problem.contest.as_str()),
                ("source", code),
                ("tabsize", "4"),
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

    pub fn logout(&self) -> Result<()> {
        block_on(async {
            self.client
                .get("https://codeforces.com/logout")
                .send()
                .await?;
            Ok(())
        })
    }
}
