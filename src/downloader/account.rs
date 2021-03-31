extern crate serde;
extern crate serde_yaml;
extern crate tokio;

use crate::{
    config::register::{HANDLE_LEN, PASSWORD_LEN, REGISTER_DELAY},
    email::{self, Email},
    judge::{self, Session},
    random::random_hex,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    vec::Vec,
};
use tokio::time::sleep;

#[derive(Debug)]
pub enum RegisterError {
    Email(email::Error),
    Judge(judge::Error),
}
impl fmt::Display for RegisterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterError::Email(err) => write!(f, "Error getting email address: {}", err),
            RegisterError::Judge(err) => write!(f, "Error registering: {}", err),
        }
    }
}
impl Error for RegisterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RegisterError::Email(err) => Some(err),
            RegisterError::Judge(err) => Some(err),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub handle: String,
    pub password: String,
    pub proxy: Option<String>,
}

pub fn from_reader<R: Read>(rdr: R) -> Result<Vec<Account>, serde_yaml::Error> {
    serde_yaml::from_reader(rdr)
}
pub async fn register(
    count: usize,
) -> (Option<(Vec<Account>, Vec<Session>)>, Option<RegisterError>) {
    let mut email = Email::new();
    if let Err(e) = email.init().await {
        return (None, Some(RegisterError::Email(e)));
    }
    let mut vec_acc = Vec::with_capacity(count);
    let mut vec_ses = Vec::with_capacity(count);
    let mut client = Session::new();
    for _ in 0..count {
        if let Err(e) = email.new_address().await {
            return (Some((vec_acc, vec_ses)), Some(RegisterError::Email(e)));
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
                return (Some((vec_acc, vec_ses)), Some(RegisterError::Judge(e)));
            }
        }
        sleep(REGISTER_DELAY).await;
    }
    (Some((vec_acc, vec_ses)), None)
}
pub fn to_writer<W: Write>(wdr: W, list: &[Account]) -> Result<(), serde_yaml::Error> {
    serde_yaml::to_writer(wdr, list)
}
