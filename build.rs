use chrono::Local;
use std::{env, println, process::Command};

fn set_env(key: &str, val: &str) {
    println!("cargo:rustc-env={}={}", key, val);
}
fn to_string(val: Vec<u8>) -> String {
    String::from_utf8(val).unwrap()
}
fn exec_args<I: std::iter::IntoIterator<Item = impl AsRef<std::ffi::OsStr>>>(
    cmd: &str,
    args: I,
) -> String {
    to_string(Command::new(cmd).args(args).output().unwrap().stdout)
}
fn exec(cmd: &str) -> String {
    to_string(Command::new(cmd).output().unwrap().stdout)
}

fn main() {
    set_env(
        "GIT_HASH",
        exec_args("git", &["log", "-1", "--pretty=format:%H"]).trim(),
    );
    {
        let branch = exec_args("git", &["symbolic-ref", "--short", "-q", "HEAD"]);
        let trim = branch.trim();
        if trim.is_empty() {
            set_env(
                "GIT_BRANCH",
                exec_args("git", &["describe", "--tags", "--exact-match", "HEAD"]).trim(),
            );
        } else {
            set_env("GIT_BRANCH", trim);
        }
    }
    set_env("BUILD_TYPE", env::var("PROFILE").unwrap().as_str());
    set_env("BUILD_HOST", exec("hostname").trim());
    set_env("BUILD_TIME", Local::now().to_rfc3339().as_str());
    set_env("CARGO", exec_args("cargo", &["version"]).trim());
    set_env("RUSTC", exec_args("rustc", &["-V"]).trim());
}
