extern crate serde;
extern crate serde_yaml;
extern crate tokio;

use crate::{
    config::register::{HANDLE_LEN, PASSWORD_LEN, REGISTER_DELAY},
    email::Email,
    judge::session::Session,
    random::random_string,
    types::Result,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    vec::Vec,
};
use tokio::time::sleep;

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub handle: String,
    pub password: String,
    pub proxy: Option<String>,
}

pub fn from_reader<R: Read>(rdr: R) -> Result<Vec<Account>> {
    Ok(serde_yaml::from_reader(rdr)?)
}
pub async fn register(count: usize) -> Result<Vec<Account>> {
    let mut ret = Vec::with_capacity(count);
    let mut email = Email::new();
    let client = Session::new();
    email.init().await?;
    for _ in 0..count {
        let cur = Account {
            handle: random_string(HANDLE_LEN),
            password: random_string(PASSWORD_LEN),
            proxy: None,
        };
        client
            .register(cur.handle.as_str(), cur.password.as_str(), &email)
            .await?;
        ret.push(cur);
        email.new_address().await?;
        sleep(REGISTER_DELAY).await;
    }
    Ok(ret)
}
pub fn to_writer<W: Write>(wdr: W, list: &Vec<Account>) -> Result<()> {
    serde_yaml::to_writer(wdr, list)?;
    Ok(())
}
