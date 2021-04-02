extern crate termcolor;

use cf_downloader::{
    encoding::Template,
    judge::{
        problem::{Problem, Type},
        Session,
    },
};
use std::{
    fs::File,
    io::{self, stdin, Read, Write},
};
use termcolor::{Color, StandardStream, WriteColor};

#[allow(unused_must_use)]
pub fn read_line_to(stdout: &mut StandardStream, prompt: &[u8], dest: &mut String) {
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
pub fn read_line(stdout: &mut StandardStream, prompt: &[u8]) -> String {
    let mut ret = String::new();
    read_line_to(stdout, prompt, &mut ret);
    ret
}
#[allow(unused_must_use)]
pub fn read_usize(stdout: &mut StandardStream, prompt: &[u8], min: usize, max: usize) -> usize {
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
pub async fn read_problem(stdout: &mut StandardStream, session: &Session) -> Problem {
    let mut contest = String::new();
    let mut id = String::new();
    loop {
        read_line_to(stdout, b"Contest: ", &mut contest);
        read_line_to(stdout, b"Problem id: ", &mut id);
        match session.check_exist(Type::Contest, &contest, &id).await {
            Ok(true) => return Problem::new(Type::Contest, contest, id),
            Ok(false) => write_error!(stdout, "Error", "No such problem or contest."),
            Err(e) => write_error!(stdout, "Error", "Check problem: {}", e.to_string()),
        }
        stdout.reset();
    }
}
#[allow(unused_must_use)]
pub fn read_template(stdout: &mut StandardStream) -> Template {
    let lang = read_line(stdout, b"Language: ");
    let mut path = String::new();
    let mut content = String::new();
    loop {
        read_line_to(stdout, b"File path: ", &mut path);
        match File::open(&path).and_then(|mut f: File| f.read_to_string(&mut content)) {
            Ok(_) => {
                break Template {
                    language: lang,
                    content,
                };
            }
            Err(e) => write_error!(stdout, "Error", "read file: {}", e.to_string()),
        }
        stdout.reset();
    }
}

#[allow(unused_must_use)]
fn read_file_path<F: Fn(&String) -> Result<Ret, io::Error>, Ret>(
    stdout: &mut StandardStream,
    fun: F,
) -> Ret {
    let mut path = String::new();
    loop {
        read_line_to(stdout, b"File path: ", &mut path);
        match fun(&path) {
            Ok(v) => break v,
            Err(e) => write_error!(stdout, "Error", "Error open {}: {}", path, e),
        }
    }
}

#[allow(unused_must_use)]
pub fn read_reader(stdout: &mut StandardStream) -> impl Read {
    read_file_path(stdout, |x| File::open(x))
}

#[allow(unused_must_use)]
pub fn read_writer(stdout: &mut StandardStream) -> impl Write {
    read_file_path(stdout, |x| File::create(x))
}
