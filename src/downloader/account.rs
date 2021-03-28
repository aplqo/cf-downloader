extern crate serde;
extern crate serde_yaml;
extern crate tokio;

use crate::{
    config::register::{HANDLE_LEN, PASSWORD_LEN, REGISTER_DELAY},
    email::Email,
    judge::session::Session,
    random::random_hex,
    types::Result,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
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
pub async fn register(
    count: usize,
) -> (Option<(Vec<Account>, Vec<Session>)>, Option<Box<dyn Error>>) {
    let mut email = Email::new();
    if let Err(e) = email.init().await {
        return (None, Some(e));
    }
    let mut vec_acc = Vec::with_capacity(count);
    let mut vec_ses = Vec::with_capacity(count);
    let mut client = Session::new();
    for _ in 0..count {
        if let Err(e) = email.new_address().await {
            return (Some((vec_acc, vec_ses)), Some(e));
        }
        let cur = Account {
            handle: random_hex(HANDLE_LEN),
            password: random_hex(PASSWORD_LEN),
            proxy: None,
        };
        client.handle = cur.handle.clone();
        match client.register(cur.password.as_str(), &email).await {
            Ok(_) => {
                vec_ses.push(client);
                vec_acc.push(cur);
                client = Session::new();
            }
            Err(e) => {
                return (Some((vec_acc, vec_ses)), Some(e));
            }
        }
        sleep(REGISTER_DELAY).await;
    }
    (Some((vec_acc, vec_ses)), None)
}
pub fn to_writer<W: Write>(wdr: W, list: &Vec<Account>) -> Result<()> {
    serde_yaml::to_writer(wdr, list)?;
    Ok(())
}
