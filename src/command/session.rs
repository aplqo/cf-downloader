extern crate termcolor;

use crate::read::{read_line, read_usize};
use cf_downloader::{
    account::{self, Account},
    judge::Session,
    submitter::{self, Submitter},
};
use std::{
    fs::File,
    io::{Read, Write},
};
use termcolor::{Color, StandardStream};

#[allow(unused_must_use)]
pub async fn login<R: Read>(stdout: &mut StandardStream, submitter: &mut Submitter, rdr: R) {
    write_info!(stdout, "Info", "Logging in...");
    let err: Result<Vec<submitter::Error>, ()> = try {
        submitter
            .login(account::from_reader(rdr).map_err(|e| {
                write_error!(stdout, "Error", "Error load account: {}", e);
            })?)
            .await
    };
    if let Ok(e) = err {
        if e.is_empty() {
            write_ok!(stdout, "Success", "Logged into codeforces.com");
        } else {
            e.into_iter()
                .for_each(|e| write_error!(stdout, "Error", "login: {}", e));
        }
    }
}

#[allow(unused_must_use)]
fn write_account<W: Write>(stdout: &mut StandardStream, wdr: W, account: Vec<Account>) {
    match account::to_writer(wdr, &account) {
        Ok(_) => write_ok!(stdout, "Success", "Written to file"),
        Err(e) => write_error!(stdout, "Error", "Error write file: {}", e),
    }
}
#[allow(unused_must_use)]
pub async fn register(stdout: &mut StandardStream) -> Option<Vec<Session>> {
    let count = read_usize(stdout, b"Count: ", 1, usize::MAX);
    let wdr = match File::create(read_line(stdout, b"File path: ")) {
        Ok(f) => f,
        Err(e) => {
            write_error!(stdout, "Error", "Error crate file: {}", e);
            return None;
        }
    };
    write_info!(stdout, "Info", "Registering {} account...", count);
    match account::register(count).await {
        (None, Some(e)) => {
            write_error!(stdout, "Error", "{}", e);
            None
        }
        (Some((acc, ses)), None) => {
            write_ok!(stdout, "Success", "Registered {} account", count);
            write_account(stdout, wdr, acc);
            Some(ses)
        }
        (Some((acc, ses)), Some(e)) => {
            write_ok!(stdout, "Finished", "Registered {} account", count);
            write_error!(stdout, "Error", "register: {}", e);
            write_account(stdout, wdr, acc);
            Some(ses)
        }
        _ => panic!("Unexpected register result"),
    }
}
#[allow(unused_must_use)]
pub async fn logout(stdout: &mut StandardStream, submitter: &mut Submitter) {
    write_info!(stdout, "Info", "Logging out from codeforces.com");
    let v = submitter.logout().await;
    if v.is_empty() {
        write_ok!(stdout, "Success", "Logged out from codeforces.com");
    } else {
        v.into_iter()
            .for_each(|e| write_error!(stdout, "Error", "logout: {}", e))
    }
}
