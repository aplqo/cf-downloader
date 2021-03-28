extern crate regex;
extern crate reqwest;

use super::{
    retry::async_retry,
    search::{search_response_or, search_text_or},
    UtilityRegex,
};
use crate::{
    account::Account,
    config::judge::session::{BFAA, VERBOSE},
    random::random_hex,
    types::{Error, Result},
};
use regex::Regex;
use reqwest::{Client, ClientBuilder, Proxy};

const FIREFOX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

pub(super) struct RegexSet {
    csrf: Regex,
    login: Regex,
    logout: Regex,
}
impl RegexSet {
    pub(super) fn new() -> Self {
        RegexSet {
            csrf: Regex::new("csrf='(.+)'").unwrap(),
            login: Regex::new(r#"handle = "[[:word:]]+"#).unwrap(),
            logout: Regex::new(r#"<a href="/([[:xdigit:]]+)/logout""#).unwrap(),
        }
    }
}

pub struct Session {
    pub(super) client: reqwest::Client,
    pub handle: String,
    pub(super) ftaa: String,
    pub(super) regex: UtilityRegex,
}
impl Session {
    fn from_client(builder: ClientBuilder) -> Self {
        Session {
            client: builder
                .user_agent(FIREFOX_UA)
                .cookie_store(true)
                .connection_verbose(VERBOSE)
                .build()
                .unwrap(),
            handle: String::new(),
            ftaa: random_hex(18),
            regex: UtilityRegex::new(),
        }
    }
    pub fn new() -> Self {
        Self::from_client(Client::builder())
    }
    pub async fn from_account(login: Account) -> Result<Self> {
        let ret = if let Some(p) = login.proxy {
            Self::from_client(Client::builder().proxy(Proxy::https(p)?))
        } else {
            Self::new()
        };
        ret.login(login.password.as_str()).await?;
        Ok(ret)
    }

    pub(super) fn find_csrf(&self, response: &String) -> Result<String> {
        search_text_or(
            response,
            &self.regex.session.csrf,
            "Regex to find csrf token not matched",
        )
    }
    pub(super) async fn get_csrf(&self, url: &str) -> Result<String> {
        self.find_csrf(&self.client.get(url).send().await?.text().await?)
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
        if self.regex.session.login.is_match(body.as_str()) {
            Ok(())
        } else {
            Err(Error::new(String::from(
                "Failed to log into codeforces.com",
            )))
        }
    }
    pub async fn logout(&self) -> Result<()> {
        async_retry(async || {
            self.client
                .get(format!(
                    "https://codeforces.com/{}/logout",
                    search_response_or(
                        || self.client.get("https://codeforces.com"),
                        &self.regex.session.logout,
                        "Logout url regex mismatch",
                    )
                    .await?
                ))
                .send()
                .await?
                .error_for_status()
        })
        .await
        .map_err(Error::from)
    }
}
