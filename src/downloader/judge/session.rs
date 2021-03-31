extern crate regex;
extern crate reqwest;

use super::{
    error::{network_error, regex_mismatch, Error, Kind, Result},
    retry::async_retry,
    search::{search_response, search_text},
    UtilityRegex,
};
use crate::{
    config::judge::session::{BFAA, VERBOSE},
    random::random_hex,
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

fn csrf_network_error(error: reqwest::Error) -> Error {
    Error::new(Kind::CSRF(network_error(error)), None)
}

pub struct Session {
    pub(super) client: reqwest::Client,
    pub handle: String,
    pub online: bool,
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
            online: false,
            ftaa: random_hex(18),
            regex: UtilityRegex::new(),
        }
    }
    pub fn new() -> Self {
        Self::from_client(Client::builder())
    }
    pub fn with_proxy(proxy: Option<String>) -> Result<Self> {
        Ok(if let Some(p) = proxy {
            Self::from_client(
                Client::builder()
                    .proxy(Proxy::https(p))
                    .map_err(|x| Error::new(Kind::Builder(x), None))?,
            )
        } else {
            Self::new()
        })
    }

    pub(super) fn find_csrf(&self, response: &String) -> Result<String> {
        search_text(response, &self.regex.session.csrf)
            .ok_or_else(|| Error::new(Kind::CSRF(regex_mismatch(None)), None))
    }
    pub(super) async fn get_csrf(&self, url: &str) -> Result<String> {
        self.find_csrf(
            &self
                .client
                .get(url)
                .send()
                .await
                .map_err(csrf_network_error)?
                .text()
                .await
                .map_err(csrf_network_error)?,
        )
    }

    pub async fn login(&mut self, handle: String, password: &str) -> Result<()> {
        const URL: &str = "https://codeforces.com/enter";
        self.handle = handle;
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
        .await
        .map_err(network_error)?;
        if self.regex.session.login.is_match(body.as_str()) {
            self.online = true;
            Ok(())
        } else {
            Err(Error::with_description(
                Kind::API,
                "Failed to login to codeforces.com",
            ))
        }
    }
    pub async fn logout(&mut self) -> Result<()> {
        if !self.online {
            return Ok(());
        }
        let url = search_response(
            || self.client.get("https://codeforces.com"),
            &self.regex.session.logout,
        )
        .await?
        .ok_or_else(|| Error::new(Kind::Regex, Some(String::from("Can't find logout url"))))?;
        async_retry(async || {
            self.client
                .get(format!("https://codeforces.com/{}/logout", url))
                .send()
                .await?
                .error_for_status()
        })
        .await
        .map_err(network_error)?;
        self.online = false;
        Ok(())
    }
}
