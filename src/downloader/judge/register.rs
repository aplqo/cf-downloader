extern crate regex;
use regex::Regex;

use super::{retry::async_retry, search::search_text_or, session::Session};
use crate::{
    config::judge::session::BFAA,
    email::Email,
    types::{Error, Result},
};

pub(super) struct RegexSet {
    name: Regex,
    error: Regex,
}
impl RegexSet {
    fn new() -> Self {
        Self {
            name: Regex::new(r#"\$\("input\[name=name\]"\)\.val\("([[:xdigit:]]+)"\);"#).unwrap(),
            error: Regex::new(r#"error for__[[:alpha:]]+">([[:alnum:][:blank:]]+)</span>"#)
                .unwrap(),
        }
    }
    fn find_error(&self, body: &String) -> Option<String> {
        let ret = self
            .error
            .captures_iter(body.as_str())
            .fold(String::new(), |text, v| {
                text.push_str(v.get(1).unwrap().as_str());
                text.push(';');
                text
            });
        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }
    fn find_name(&self, response: &String) -> Result<String> {
        search_text_or(response.as_str(), &self.name, "Can't find register name")
    }
}

impl Session {
    async fn post_empty(&self, ftaa: &str, csrf: &str) -> Result<()> {
        async_retry(async || {
            self.client
                .post("https://codeforces.com/data/empty")
                .form(&[("bfaa", BFAA), ("ftaa", ftaa), ("csrf_token", csrf)])
                .send()
                .await?
                .error_for_status()
        })
        .await?;
        Ok(())
    }
    pub async fn register(&self, password: &str, email: &Email) -> Result<()> {
        let regex = RegexSet::new();
        const URL: &str = "https://codeforces.com/register";
        let body: String = async_retry(async || {
            self.client
                .get(URL)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await
        })
        .await?;
        let csrf = self.find_csrf(&body)?;
        let name = regex.find_name(&body)?;
        async_retry(async || {
            self.client
                .post(URL)
                .form(&[("action", "welcome"), ("csrf_token", csrf.as_str())])
                .send()
                .await?
                .error_for_status()
        })
        .await?;
        self.post_empty("", csrf.as_str()).await?;
        self.post_empty(self.ftaa.as_str(), csrf.as_str()).await?;

        regex
            .find_error(
                &async_retry(async || {
                    self.client
                        .post(URL)
                        .form(&[
                            ("csrf_token", csrf.as_str()),
                            ("ftaa", self.ftaa.as_str()),
                            ("bfaa", BFAA),
                            ("action", "register"),
                            ("handle", self.handle.as_str()),
                            ("name", name.as_str()),
                            ("age", ""),
                            ("email", email.address.as_str()),
                            ("password", password),
                            ("passwordConfirmation", password),
                            ("_tta", "510"),
                        ])
                        .send()
                        .await?
                        .error_for_status()?
                        .text()
                        .await
                })
                .await?,
            )
            .map_or(Ok(()), |x| Err(Error::new(x)))?;
        for p in email.wait_email_urls("noreply@codeforces.com").await? {
            if p.contains("register") {
                async_retry(async || self.client.get(p.as_str()).send().await?.error_for_status())
                    .await?;
                return Ok(());
            }
        }
        Err(Error::new("Can't find confirm address".to_string()))
    }
}
