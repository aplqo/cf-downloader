extern crate termcolor;

use crate::{
    read::{read_line, read_problem, read_reader, read_template, read_usize, read_writer},
    write::write_result,
};
use cf_downloader::{
    downloader::{data::DataResult, Downloader},
    encoding::{
        gzip::Decoder,
        handlebars::{encode::Encoder, meta::Meta},
    },
    judge::Session,
    submitter::Submitter,
};
use std::{fs::File, io::Write};
use termcolor::{Color, StandardStream, WriteColor};

#[allow(unused_must_use)]
async fn get_data(stdout: &mut StandardStream, downloader: &mut Downloader<'_>) {
    if downloader.is_empty() {
        write_error!(stdout, "Error", "No metadata");
        return;
    }
    let begin = read_usize(stdout, b"Begin: ", 0, downloader.len());
    let end = read_usize(stdout, b"End: ", begin + 1, downloader.len() + 1);
    match downloader
        .get_data::<Encoder, Decoder, _>(&read_template(stdout), begin, end)
        .await
    {
        DataResult::Build(e) => write_error!(stdout, "Fail", "{}", e),
        DataResult::Result(v) => {
            if v.is_empty() {
                write_ok!(stdout, "Finish", "Got {} data", end - begin);
            }
            for (index, val) in (begin..end).zip(v.into_iter()) {
                match val {
                    Ok(v) => {
                        if let Err(e) = File::create(format!("{}.in", index))
                            .and_then(|mut f: File| f.write(v.as_bytes()))
                        {
                            write_error!(stdout, "Fail", "write data {}: {}", index, e);
                        }
                    }
                    Err(e) => {
                        write_error!(stdout, "Error", "fail get test {}: {}", index, e);
                    }
                }
            }
        }
    };
}

#[allow(unused_must_use)]
async fn get_meta(stdout: &mut StandardStream, downloader: &mut Downloader<'_>) {
    let cnt = read_usize(stdout, b"Until: ", 0, usize::MAX);
    let template = read_template(stdout);
    write_info!(stdout, "Info", "Loading {} more testcase's metadata", cnt);
    if let Err(e) = downloader.get_meta::<Meta, _>(&template, cnt).await {
        write_error!(stdout, "Fail", "{}", e.to_string());
    } else {
        write_ok!(stdout, "Success", "Successfully getted metadata");
    }
}

#[allow(unused_must_use)]
pub async fn problem_loop(
    stdout: &mut StandardStream,
    session: &Session,
    submitter: &'_ mut Submitter,
) {
    let problem = read_problem(stdout, session).await;
    write_info!(
        stdout,
        "Info",
        "Selected problem {}{}",
        problem.contest,
        problem.id
    );
    stdout.reset();
    let prompt = format!("cf-downloader [{} {}]> ", problem.contest, problem.id);
    let mut downloader: Downloader = Downloader::new(problem, submitter);
    let stdout_ptr: *mut StandardStream = stdout;
    loop {
        match read_line(stdout, prompt.as_bytes()).trim() {
            "get_meta" => get_meta(stdout, &mut downloader).await,
            "unselect" => {
                write_info!(stdout, "Info", "Unselected problem");
                break;
            }
            "get_data" => get_data(stdout, &mut downloader).await,
            "load_meta" => write_result(
                stdout,
                downloader.load_meta(read_reader(unsafe { &mut *stdout_ptr })),
                "Loaded metadata",
            ),
            "save_meta" => write_result(
                stdout,
                downloader.save_meta(read_writer(unsafe { &mut *stdout_ptr })),
                "Written metadata to file",
            ),
            "load_cache" => write_result(
                stdout,
                downloader
                    .cache
                    .load(read_reader(unsafe { &mut *stdout_ptr })),
                "Loaded cache from file",
            ),
            "flush_cache" => {
                downloader.cache.flush();
                write_ok!(stdout, "Success", "Flushed cache");
            }
            unknown => write_error!(stdout, "Error", "problem: Unknown command {}", unknown),
        }
        stdout.reset();
    }
    stdout.reset();
}
