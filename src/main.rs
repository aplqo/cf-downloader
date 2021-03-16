extern crate clap;
extern crate termcolor;

use cf_downloader::{
    client::{Problem, ProblemType, Session},
    downloader::Downloader,
    encoding::{
        gzip::Decoder,
        handlebars::{encode, meta},
        Template,
    },
    types::Result,
};
use clap::Clap;
use std::{
    fs::File,
    io::{stdin, Read, Write},
    path::Path,
    writeln,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::runtime::Runtime;

#[derive(Clap)]
struct Login {
    handle: String,
    password: String,
    proxy: String,
}

macro_rules! write_color {
    ($dest:expr, $color:expr, $($arg:tt)*) => { {
        $dest.set_color(ColorSpec::new().set_fg(Some($color))).expect("Failed to set output color");
        writeln!($dest, $($arg)*).expect("Failed to write output");
    }
    };
}
macro_rules! write_error {
    ($dest:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Red, $($arg)*);
    };
}
macro_rules! write_info {
    ($dest:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Cyan, $($arg)*);
    };
}
macro_rules! write_ok {
    ($dest:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Green, $($arg)*);
    };
}

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
            Err(e) => write_error!(stdout, "Read error: {}", e.to_string()),
        }
        stdout.reset();
    }
}
fn read_line(stdout: &mut StandardStream, prompt: &[u8]) -> String {
    let mut ret = String::new();
    read_line_to(stdout, prompt, &mut ret);
    return ret;
}
fn read_usize(stdout: &mut StandardStream, prompt: &[u8], min: usize, max: usize) -> usize {
    let mut buf = String::new();
    loop {
        read_line_to(stdout, prompt, &mut buf);
        match buf.parse::<usize>() {
            Ok(v) => {
                if v < min || v >= max {
                    write_error!(
                        stdout,
                        "Value {} out of range. Expected value in [{}, {})",
                        v,
                        min,
                        max
                    );
                } else {
                    return v;
                }
            }
            Err(e) => write_error!(stdout, "Parse error: {}", e.to_string()),
        }
        stdout.reset();
    }
}
fn read_problem(stdout: &mut StandardStream, session: &Session, rt: &Runtime) -> Problem {
    let mut ret = Problem {
        source: ProblemType::Contest,
        contest: String::new(),
        id: String::new(),
    };
    loop {
        read_line_to(stdout, b"Enter contest: ", &mut ret.contest);
        read_line_to(stdout, b"Enter problem id: ", &mut ret.id);
        match rt.block_on(async { session.check_exist(&ret).await }) {
            Ok(true) => return ret,
            Ok(false) => write_error!(stdout, "No such problem or contest."),
            Err(e) => write_error!(stdout, "Error checking problem: {}", e.to_string()),
        }
        stdout.reset();
    }
}
fn read_template(stdout: &mut StandardStream) -> Template {
    let lang = read_line(stdout, b"Enter language: ");
    let mut path = String::new();
    let mut content = String::new();
    loop {
        read_line_to(stdout, b"Enter file path: ", &mut path);
        match File::open(&path).and_then(|mut f: File| f.read_to_string(&mut content)) {
            Ok(_) => {
                return Template {
                    language: lang,
                    content: content,
                };
            }
            Err(e) => write_error!(stdout, "Error read file {}", e.to_string()),
        }
    }
}

fn problem_loop(stdout: &mut StandardStream, session: &Session, rt: &Runtime) {
    let problem = read_problem(stdout, session, rt);
    write_ok!(stdout, "Selected problem {}{}", problem.contest, problem.id);
    stdout.reset();
    let mut downloader: Downloader = Downloader::new(session, &problem);
    let prompt = format!("cf-downloader [{} {}]> ", problem.contest, problem.id);
    loop {
        match read_line(stdout, prompt.as_bytes()).trim() {
            "get_meta" => {
                let cnt = read_usize(stdout, b"Enter count:  ", 1, usize::MAX);
                let template = read_template(stdout);
                write_info!(stdout, "Loading {} more testcase's metadata", cnt);
                if let Err(e) = rt.block_on(downloader.get_meta::<meta::Meta>(&template, cnt)) {
                    write_error!(stdout, "{}", e.to_string());
                } else {
                    write_ok!(stdout, "Successfully getted metadata");
                }
            }
            "unselect" => {
                break;
            }
            "get_data" => {
                if downloader.testdata.is_empty() {
                    write_error!(stdout, "No metadata");
                } else {
                    let begin = read_usize(stdout, b"begin: ", 0, prompt.len());
                    let end =
                        read_usize(stdout, b"end: ", begin + 1, downloader.testdata.len() + 1);
                    match rt.block_on(async {
                        downloader
                            .get_data::<encode::Encoder, Decoder>(
                                &read_template(stdout),
                                begin,
                                end,
                            )
                            .await
                    }) {
                        Ok(v) => {
                            for i in begin..end {
                                if let Err(e) = File::create(i.to_string())
                                    .and_then(|mut f: File| f.write(v[i - begin].as_bytes()))
                                {
                                    write_error!(
                                        stdout,
                                        "Error in writing file: {}",
                                        e.to_string()
                                    );
                                }
                            }
                        }
                        Err(e) => write_error!(stdout, "Failed getting data: {}", e.to_string()),
                    };
                }
            }
            "load" => {
                match downloader
                    .load_meta(Path::new(read_line(stdout, b"Enter file path: ").as_str()))
                {
                    Ok(_) => write_ok!(stdout, "Successfully loaded metadata"),
                    Err(e) => write_error!(stdout, "Error loading metadata: {}", e.to_string()),
                }
            }
            "save" => {
                match downloader
                    .save_meta(Path::new(read_line(stdout, b"Enter file path").as_str()))
                {
                    Ok(_) => write_ok!(stdout, "Successfully writed metadata to file"),
                    Err(e) => write_error!(stdout, "Error writing metadata: {}", e.to_string()),
                }
            }
            unknown => write_error!(stdout, "Unknown command {}", unknown),
        }
        stdout.reset();
    }
}
async fn login(login: Login) -> Result<Session> {
    let ret = Session::new(login.handle, login.proxy.as_str())?;
    ret.login(login.password.as_str()).await?;
    Ok(ret)
}

fn main() {
    let rt = Runtime::new().unwrap();
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let info: Login = Login::parse();
    write_info!(
        &mut stdout,
        "Loging into codeforces.com as {} proxy: {}",
        info.handle,
        info.proxy
    );

    let session = match rt.block_on(login(info)) {
        Ok(v) => {
            write_ok!(&mut stdout, "Logged into codeforces.com");
            v
        }
        Err(e) => {
            write_error!(&mut stdout, "Failed to login: {}", e.to_string());
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
            unknown => write_error!(&mut stdout, r#"unknown command "{}""#, unknown),
        }
        stdout.reset();
    }

    write_info!(&mut stdout, "Logging out from codeforces.com");
    rt.block_on(session.logout()).unwrap();
    write_ok!(&mut stdout, "Logged out from codeforces.com");
    stdout.reset();
}
