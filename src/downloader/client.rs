extern crate regex;
extern crate reqwest;

use crate::types::Error;
use futures::executor::block_on;
use regex::Regex;
use reqwest::{Client, Proxy};

const BFAA: &str = "f1b3f18c715565b589b7823cda7448ce";
const FIREFOX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

pub enum ProblemType {
    Contest,
    Gym,
}
pub struct Problem {
    pub source: ProblemType,
    pub contest: String,
    id: String,
}
pub struct Submission {
    pub output: String,
    pub answer: Option<String>,
}
struct UtilityRegex {
    csrf: Regex,
    login: Regex,
    submit: Regex,
}
pub struct Session {
    client: Client,
    handle: String,
    ftaa: String,
    regex: UtilityRegex,
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error {
            description: e.to_string(),
        }
    }
}

impl UtilityRegex {
    fn new() -> Self {
        UtilityRegex {
            csrf: Regex::new("csrf='(.+?)'").unwrap(),
            login: Regex::new(r#"handle = "([\s\S]+?)"#).unwrap(),
            submit: Regex::new(r#"error[a-zA-Z_\-\ ]*">(.*?)</span>"#).unwrap(),
        }
    }
}

impl Session {
    pub fn new(handle: String, proxy: &str) -> Result<Self, String> {
        let proxy = match Proxy::https(proxy) {
            Ok(p) => p,
            Err(e) => return Err(e.to_string()),
        };
        Ok(Session {
            client: Client::builder()
                .proxy(proxy)
                .user_agent(FIREFOX_UA)
                .cookie_store(true)
                .build()
                .unwrap(),
            handle: handle,
            ftaa: "123456789123456789".to_string(),
            regex: UtilityRegex::new(),
        })
    }

    async fn get_csrf(&self, url: &str) -> Result<String, reqwest::Error> {
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

    pub fn login(&self, password: &str) -> Result<(), Error> {
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
            Err(Error {
                description: "Failed to log into codeforces.com".to_string(),
            })
        }
    }
    pub async fn get_last_submission_url(&self, problem: &Problem) -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn wait_submission(&self, url: &String) -> Result<Submission, Error> {
        Err(Error {
            description: String::new(),
        })
    }
    pub async fn submit(&self, problem: &Problem, language: &str, code: &str) -> Result<(), Error> {
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
            Some(err) => Err(Error {
                description: err.get(1).unwrap().as_str().to_string(),
            }),
            None => Ok(()),
        }
    }

    pub fn logout(&self) -> Result<(), String> {
        match block_on(async {
            self.client
                .get("https://codeforces.com/logout")
                .send()
                .await
        }) {
            Ok(_) => Ok(()),
            Err(e) => return Err(e.to_string()),
        }
    }
}
