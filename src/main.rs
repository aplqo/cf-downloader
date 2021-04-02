#![feature(try_blocks)]
#![feature(nll)]
extern crate clap;
extern crate pretty_env_logger;
extern crate termcolor;
extern crate tokio;

use cf_downloader::{judge::Session, submitter::Submitter};
use clap::{crate_description, crate_name, App, Arg};
use pretty_env_logger::init_timed;
use std::{fs::File, io::Write};
use termcolor::{Color, ColorChoice, StandardStream, WriteColor};

#[macro_use]
mod color;
mod command {
    pub mod problem;
    pub mod session;
}
mod read;
mod write;

use command::{
    problem::problem_loop,
    session::{login, logout, register},
};
use read::{read_line, read_reader};

#[allow(unused_must_use)]
#[tokio::main]
async fn main() {
    init_timed();
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let app = App::new(crate_name!())
        .about(crate_description!())
        .version(get_version!("version"))
        .long_version(get_version!("long_version"))
        .arg(Arg::new("account").about("Path to account list"))
        .get_matches();
    let session = Session::new();
    let mut submit = Submitter::new();
    if let Some(f) = app.value_of("account") {
        match File::open(f) {
            Ok(v) => login(&mut stdout, &mut submit, v).await,
            Err(e) => write_error!(&mut stdout, "Error", "Error open {}: {}", f, e),
        }
        stdout.reset();
    }
    let stdout_ptr: *mut StandardStream = &mut stdout;
    loop {
        match read_line(&mut stdout, b"cf-downloader> ").trim() {
            "select" => {
                if submit.is_empty() {
                    write_error!(&mut stdout, "Error", "No logined account!");
                } else {
                    problem_loop(&mut stdout, &session, &mut submit).await;
                }
            }
            "exit" => break,
            "login" => {
                login(
                    &mut stdout,
                    &mut submit,
                    read_reader(unsafe { &mut *stdout_ptr }),
                )
                .await
            }
            "register" => {
                if let Some(v) = register(&mut stdout).await {
                    submit.add_session(v);
                }
            }
            "logout" => logout(&mut stdout, &mut submit).await,
            unknown => write_error!(
                &mut stdout,
                "Error",
                r#"cf-downloader: unknown command "{}""#,
                unknown
            ),
        }
        stdout.reset();
    }
    logout(&mut stdout, &mut submit).await;
    stdout.reset();
}
