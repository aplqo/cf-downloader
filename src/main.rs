extern crate clap;
extern crate termcolor;

use cf_downloader::{
    client::{Problem, ProblemType, Session},
    downloader::{Callback, Downloader, CHECK_DELAY, SUBMISSION_GET_DELAY, SUBMIT_DELAY},
    encoding::{
        gzip::Decoder,
        handlebars::{encode::Encoder, meta::Meta},
        Template,
    },
    types::Result,
};
use clap::Clap;
use std::{
    fs::File,
    io::{stdin, Read, Write},
    path::Path,
    println, writeln,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::runtime::Runtime;

#[derive(Clap)]
struct Login {
    handle: String,
    password: String,
    proxy: String,
}

fn set_fg(stdout: &mut StandardStream, color: Color) {
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)).set_intense(true))
        .expect("Error: can't set output color");
}
fn reset_fg(stdout: &mut StandardStream) {
    stdout
        .set_color(ColorSpec::new().set_fg(None).set_intense(true))
        .expect("Error: Can't reset color");
}
macro_rules! write_color {
    ($dest:expr, $color:expr,$typ:expr,  $($arg:tt)*) => { {
        set_fg($dest, $color);
        write!($dest,"{:>8}: ", $typ);
        reset_fg($dest);
        writeln!($dest, $($arg)*).expect("Failed to write output");
    }
    };
}
macro_rules! write_error {
    ($dest:expr,$typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Red, $typ, $($arg)*);
    };
}
macro_rules! write_info {
    ($dest:expr,$typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Blue, $typ, $($arg)*);
    };
}
macro_rules! write_ok {
    ($dest:expr,$typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Green, $typ, $($arg)*);
    };
}
macro_rules! write_progress {
    ($dest:expr, $typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Cyan, $typ, $($arg)*);
    };
}

#[allow(unused_must_use)]
fn read_line_to(stdout: &mut StandardStream, prompt: &[u8], dest: &mut String) {
    dest.clear();
    loop {
        stdout.write(prompt);
        stdout.flush();
        match stdin().read_line(dest) {
            Ok(_) => {
                dest.truncate(dest.trim_end().len());
                return;
            }
            Err(e) => write_error!(stdout, "Error", "Read: {}", e.to_string()),
        }
        stdout.reset();
    }
}
fn read_line(stdout: &mut StandardStream, prompt: &[u8]) -> String {
    let mut ret = String::new();
    read_line_to(stdout, prompt, &mut ret);
    return ret;
}
#[allow(unused_must_use)]
fn read_usize(stdout: &mut StandardStream, prompt: &[u8], min: usize, max: usize) -> usize {
    let mut buf = String::new();
    loop {
        read_line_to(stdout, prompt, &mut buf);
        match buf.parse::<usize>() {
            Ok(v) => {
                if v < min || v >= max {
                    write_error!(
                        stdout,
                        "Error",
                        "parse: Value {} out of range. Expected value in [{}, {})",
                        v,
                        min,
                        max
                    );
                } else {
                    return v;
                }
            }
            Err(e) => write_error!(stdout, "Error", "parse: {}", e.to_string()),
        };
        stdout.reset();
    }
}
#[allow(unused_must_use)]
fn read_problem(stdout: &mut StandardStream, session: &Session, rt: &Runtime) -> Problem {
    let mut ret = Problem {
        source: ProblemType::Contest,
        contest: String::new(),
        id: String::new(),
    };
    loop {
        read_line_to(stdout, b"Contest: ", &mut ret.contest);
        read_line_to(stdout, b"Problem id: ", &mut ret.id);
        match rt.block_on(async { session.check_exist(&ret).await }) {
            Ok(true) => return ret,
            Ok(false) => write_error!(stdout, "Error", "No such problem or contest."),
            Err(e) => write_error!(stdout, "Error", "Get problem: {}", e.to_string()),
        }
        stdout.reset();
    }
}
#[allow(unused_must_use)]
fn read_template(stdout: &mut StandardStream) -> Template {
    let lang = read_line(stdout, b"Language: ");
    let mut path = String::new();
    let mut content = String::new();
    loop {
        read_line_to(stdout, b"File path: ", &mut path);
        match File::open(&path).and_then(|mut f: File| f.read_to_string(&mut content)) {
            Ok(_) => {
                return Template {
                    language: lang,
                    content: content,
                };
            }
            Err(e) => write_error!(stdout, "Error", "read file: {}", e.to_string()),
        }
        stdout.reset();
    }
}

struct GetMetaCall<'a> {
    stdout: &'a mut StandardStream,
}
impl<'a> Callback for GetMetaCall<'a> {
    #[allow(unused_must_use)]
    fn on_case_begin(&mut self, id: usize) {
        write_progress!(self.stdout, "Start", "Get meta data for test {}", id);
    }
    #[allow(unused_must_use)]
    fn on_case_end(&mut self, id: usize) {
        write_ok!(self.stdout, "Finish", "Got meta data for test {}", id);
    }
}
struct GetDataCall<'a> {
    stdout: &'a mut StandardStream,
}
impl<'a> Callback for GetDataCall<'a> {
    #[allow(unused_must_use)]
    fn on_case_begin(&mut self, id: usize) {
        write_progress!(self.stdout, "Start", "Get input for test {}", id);
    }
    #[allow(unused_must_use)]
    fn on_progress(&mut self, id: usize, current: usize, total: usize) {
        write_info!(
            self.stdout,
            "Info",
            "Got {} of {} data segment for test {}",
            current,
            total,
            id
        );
    }
    #[allow(unused_must_use)]
    fn on_case_end(&mut self, id: usize) {
        write_ok!(self.stdout, "Finish", "Get data for test {}", id);
    }
}

#[allow(unused_must_use)]
fn problem_loop(stdout: &mut StandardStream, session: &Session, rt: &Runtime) {
    let problem = read_problem(stdout, session, rt);
    write_info!(
        stdout,
        "Info",
        "Selected problem {}{}",
        problem.contest,
        problem.id
    );
    stdout.reset();
    let prompt = format!("cf-downloader [{} {}]> ", problem.contest, problem.id);
    let mut downloader: Downloader = Downloader::new(session, problem);
    loop {
        match read_line(stdout, prompt.as_bytes()).trim() {
            "get_meta" => {
                let cnt = read_usize(stdout, b"Count: ", 1, usize::MAX);
                let template = read_template(stdout);
                write_info!(stdout, "Info", "Loading {} more testcase's metadata", cnt);
                if let Err(e) = rt.block_on(downloader.get_meta::<Meta, _>(
                    &template,
                    cnt,
                    GetMetaCall { stdout },
                )) {
                    write_error!(stdout, "Fail", "{}", e.to_string());
                } else {
                    write_ok!(stdout, "Success", "Successfully getted metadata");
                }
            }
            "unselect" => {
                write_info!(stdout, "Info", "Unselected problem");
                break;
            }
            "get_data" => {
                if downloader.is_empty() {
                    write_error!(stdout, "Error", "No metadata");
                } else {
                    let begin = read_usize(stdout, b"Begin: ", 0, downloader.len());
                    let end = read_usize(stdout, b"End: ", begin + 1, downloader.len() + 1);
                    match rt.block_on(async {
                        downloader
                            .get_data::<Encoder, Decoder, _>(
                                &read_template(stdout),
                                begin,
                                end,
                                GetDataCall { stdout },
                            )
                            .await
                    }) {
                        Ok(v) => {
                            for i in begin..end {
                                if let Err(e) = File::create(format!("{}.in", i))
                                    .and_then(|mut f: File| f.write(v[i - begin].as_bytes()))
                                {
                                    write_error!(stdout, "Fail", "write: {}", e.to_string());
                                }
                            }
                        }
                        Err(e) => write_error!(stdout, "Fail", "get_data: {}", e.to_string()),
                    };
                }
            }
            "load" => {
                match downloader.load_meta(Path::new(read_line(stdout, b"File path: ").as_str())) {
                    Ok(_) => write_ok!(stdout, "Success", "Loaded metadata"),
                    Err(e) => write_error!(stdout, "Fail", "load: {}", e.to_string()),
                }
            }
            "save" => {
                match downloader.save_meta(Path::new(read_line(stdout, b"File path: ").as_str())) {
                    Ok(_) => write_ok!(stdout, "Success", "Writed metadata to file"),
                    Err(e) => write_error!(stdout, "Fail", "write: {}", e.to_string()),
                }
            }
            unknown => write_error!(stdout, "Error", "problem: Unknown command {}", unknown),
        }
        stdout.reset();
    }
}
async fn login(login: Login) -> Result<Session> {
    let ret = Session::new(login.handle, login.proxy.as_str())?;
    ret.login(login.password.as_str()).await?;
    Ok(ret)
}

fn print_version() {
    println!(
        "{} {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("BUILD_TYPE")
    );
    println!("version: {} git@{}", env!("GIT_BRANCH"), env!("GIT_HASH"));
    println!("build date: {}", env!("BUILD_TIME"));
    println!("build on {} with {}", env!("BUILD_HOST"), env!("RUSTC"),);
    println!(
        "Submit rate:\n\t submit delay: {}s \n\t get submission delay: {}s \n\t check delay: {}s",
        SUBMIT_DELAY.as_secs_f32(),
        SUBMISSION_GET_DELAY.as_secs_f32(),
        CHECK_DELAY.as_secs_f32()
    );
}

#[allow(unused_must_use)]
fn main() {
    print_version();
    let rt = Runtime::new().unwrap();
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let info: Login = Login::parse();
    write_info!(
        &mut stdout,
        "Info",
        "Loging into codeforces.com as {} proxy: {}",
        info.handle,
        info.proxy
    );

    let session = match rt.block_on(login(info)) {
        Ok(v) => {
            write_ok!(&mut stdout, "Success", "Logged into codeforces.com");
            v
        }
        Err(e) => {
            write_error!(&mut stdout, "Fail", "login: {}", e.to_string());
            stdout.reset();
            return;
        }
    };
    stdout.reset();

    loop {
        match read_line(&mut stdout, b"cf-downloader> ").trim() {
            "select" => {
                problem_loop(&mut stdout, &session, &rt);
            }
            "exit" => break,
            unknown => write_error!(
                &mut stdout,
                "Error",
                r#"cf-downloader: unknown command "{}""#,
                unknown
            ),
        }
        stdout.reset();
    }

    write_info!(&mut stdout, "Info", "Logging out from codeforces.com");
    rt.block_on(session.logout()).unwrap();
    write_ok!(&mut stdout, "Success", "Logged out from codeforces.com");
    stdout.reset();
}
