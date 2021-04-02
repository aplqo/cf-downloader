extern crate termcolor;

use cf_downloader::error::Error;
use std::io::Write;
use termcolor::{Color, StandardStream};

#[allow(unused_must_use)]
pub fn write_result<E: Error>(stdout: &mut StandardStream, result: Result<(), E>, success: &str) {
    match result {
        Ok(_) => write_ok!(stdout, "Success", "{}", success),
        Err(e) => write_error!(stdout, "Error", "{}", e),
    }
}
