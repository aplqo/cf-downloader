use chrono::{DateTime, Local};
use std::{env, io::Write, path::Path, process::Command};
include!("./src/downloader/config.rs");

fn launch(cmd: &mut Command) -> String {
    String::from_utf8(cmd.output().unwrap().stdout).unwrap()
}
fn exec<I: std::iter::IntoIterator<Item = impl AsRef<std::ffi::OsStr>>>(
    cmd: &str,
    args: I,
) -> String {
    launch(Command::new(cmd).args(args))
}

fn set_short_version(out_dir: &Path, date: &DateTime<Local>, branch: &str, profile: &str) {
    write!(
        std::fs::File::create(out_dir.join("version")).expect("Failed to create version file"),
        "(git@{} {} {}) {}",
        exec("git", &["log", "-1", "--pretty=format:%h"]).trim(),
        branch,
        date.date().to_string(),
        profile
    )
    .unwrap();
}
fn set_long_version(out_dir: &Path, date: &DateTime<Local>, branch: &str, profile: &str) {
    let mut f = std::io::BufWriter::new(
        std::fs::File::create(out_dir.join("long_version")).expect("Failed to create long version"),
    );
    writeln!(&mut f, "{}", profile).unwrap();
    writeln!(
        &mut f,
        "commit: {} git@{}",
        branch,
        exec("git", &["log", "-1", "--pretty=format:%H"]).trim()
    )
    .unwrap();
    writeln!(
        &mut f,
        "rustc: {} {}",
        exec(env::var("RUSTC").unwrap().as_str(), &["--version"]).trim(),
        env::var("TARGET").unwrap()
    )
    .unwrap();
    writeln!(&mut f, "date: {}", date.to_rfc3339()).unwrap();
    writeln!(
        &mut f,
        "host: {}",
        launch(&mut Command::new("hostname")).trim()
    )
    .unwrap();
    writeln!(
        &mut f,
        r#"submit_rate:
    submit_delay: {}s
    get_submisison_delay: {}s
    check_delay: {}s"#,
        submitter::SUBMIT_DELAY.as_secs_f32(),
        submitter::SUBMISSION_GET_DELAY.as_secs_f32(),
        submission::CHECK_DELAY.as_secs_f32()
    )
    .unwrap();
    writeln!(
        &mut f,
        r#"retry:
    delay: {}s
    delay_after_http403: {}s
    count: {}
    "#,
        retry::RETRY_DELAY.as_secs_f32(),
        retry::FORBIDDEN_DELAY.as_secs_f32(),
        retry::RETRY_COUNT
    )
    .unwrap();
}
fn get_branch() -> String {
    let branch = exec("git", &["symbolic-ref", "--short", "-q", "HEAD"]);
    let trim = branch.trim();
    if trim.is_empty() {
        exec("git", &["describe", "--tags", "--exact-match", "HEAD"])
            .trim()
            .to_string()
    } else {
        trim.to_string()
    }
}

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let buf = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(buf.as_str());
    let branch = get_branch();
    let time = Local::now();
    set_short_version(out_dir, &time, &branch, &profile);
    set_long_version(out_dir, &time, &branch, &profile);
}
