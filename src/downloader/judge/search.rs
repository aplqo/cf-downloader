extern crate regex;
extern crate reqwest;

use super::retry::async_retry;
use regex::Regex;
use reqwest::{RequestBuilder, Result};

pub fn search_text(text: &str, regex: &Regex) -> Option<String> {
    regex
        .captures(text)
        .map(|v| v.get(1).unwrap().as_str().to_owned())
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
