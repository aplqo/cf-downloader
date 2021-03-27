extern crate reqwest;
extern crate serde;
extern crate tokio;

use crate::{config::email::CHECK_DELAY, types::Result};
use reqwest::Client;
use serde::Deserialize;
use std::vec::Vec;
use tokio::time::sleep;
const ADDRESS_URL: &str = "https://10minutemail.net/address.api.php";
const MAIL_URL: &str = "https://10minutemail.net/mail.api.php";
const NEW_URL: &str = "https://10minutemail.net/new.html";

#[derive(Deserialize)]
struct MailEntry {
    mail_id: String,
    from: String,
}
#[derive(Deserialize)]
struct Response {
    mail_get_mail: String,
    mail_list: Vec<MailEntry>,
}

#[derive(Deserialize)]
struct Mail {
    urls: Vec<String>,
}
pub struct Email {
    client: Client,
    pub address: String,
}

impl Email {
    pub fn new() -> Self {
        Email {
            client: Client::builder().cookie_store(true).build().unwrap(),
            address: String::new(),
        }
    }
    async fn get_response(&self) -> Result<Response> {
        Ok(self
            .client
            .get(ADDRESS_URL)
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?)
    }
    pub async fn init(&mut self) -> Result<()> {
        self.address = self.get_response().await?.mail_get_mail;
        Ok(())
    }
    pub async fn wait_email_urls(&self, from: &str) -> Result<Vec<String>> {
        loop {
            for i in self.get_response().await?.mail_list {
                if i.from == from {
                    return Ok(self
                        .client
                        .get(MAIL_URL)
                        .query(&[("mailid", i.mail_id.as_str())])
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<Mail>()
                        .await?
                        .urls);
                }
            }
            sleep(CHECK_DELAY).await;
        }
    }
    pub async fn new_address(&mut self) -> Result<()> {
        self.client.get(NEW_URL).send().await?;
        self.address = self.get_response().await?.mail_get_mail;
        Ok(())
    }
}
