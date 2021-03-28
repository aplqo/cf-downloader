extern crate regex;
extern crate reqwest;

use super::retry::async_retry;
use crate::types::{Error, Result};
use regex::Regex;
use reqwest::RequestBuilder;
use std::error::Error as StdErr;

pub fn search_text(text: &str, regex: &Regex) -> Option<String> {
    regex
        .captures(text)
        .map(|v| v.get(1).unwrap().as_str().to_owned())
}
pub fn search_text_or(text: &str, regex: &Regex, error: &str) -> Result<String> {
    search_text(text, regex).ok_or_else(|| -> Box<dyn StdErr> { Error::new(error.to_string()) })
}

pub async fn search_response<T: Fn() -> RequestBuilder>(
    fun: T,
    regex: &Regex,
) -> Result<Option<String>> {
    Ok(search_text(
        &async_retry(async || fun().send().await?.error_for_status()?.text().await).await?,
        regex,
    ))
}
pub async fn search_response_or<T: Fn() -> RequestBuilder>(
    fun: T,
    regex: &Regex,
    error: &str,
) -> Result<String> {
    search_response(fun, regex)
        .await?
        .ok_or_else(|| -> Box<dyn StdErr> { Error::new(error.to_string()) })
}
