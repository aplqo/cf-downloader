extern crate tokio;

use crate::{
    config::register::{HANDLE_LEN, PASSWORD_LEN, REGISTER_DELAY},
    email::Email,
    judge::session::{Account, Session},
    random::random_string,
    types::Result,
};
use std::vec::Vec;
use tokio::time::sleep;

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
