extern crate tokio;

use super::{
    error::{Error, Kind, Operate, Result},
    Submitter,
};
use crate::{
    config::submitter::SUBMISSION_GET_DELAY,
    judge::{problem::Problem, submit::Submission, Session},
};
use std::mem::{take, MaybeUninit};
use tokio::{
    task::{spawn_local, JoinHandle},
    time::sleep,
};

async fn get_last_submission<'a>(session: &'a Session, problem: &'a Problem) -> Result<Submission> {
    sleep(SUBMISSION_GET_DELAY).await;
    session
        .get_last_submission(problem)
        .await
        .map_err(|err| Error {
            operate: Operate::GetSubmission,
            kind: Kind::Judge(err),
            handle: session.handle.clone(),
        })
}
async fn submit<'a>(
    session: &Session,
    problem: &Problem,
    language: &str,
    code: &str,
) -> Result<()> {
    session
        .submit(problem, language, code)
        .await
        .map_err(|x| Error {
            operate: Operate::Submit,
            kind: Kind::Judge(x),
            handle: session.handle.clone(),
        })
}
async fn get_result(
    handle: JoinHandle<Result<Submission>>,
    session: &Session,
) -> Result<Submission> {
    match handle.await {
        Ok(v) => v,
        Err(e) => Err(Error {
            operate: Operate::GetSubmission,
            kind: Kind::Join(e),
            handle: session.handle.clone(),
        }),
    }
}

impl Submitter {
    pub async fn submit(
        &mut self,
        problem: &Problem,
        language: &str,
        code: &str,
    ) -> Result<Submission> {
        let account = &self.session[self.list.get().await];
        submit(account, problem, language, code).await?;
        get_last_submission(account, problem).await
    }

    pub async fn submit_iter<It: IntoIterator<Item = String>>(
        &mut self,
        problem: &Problem,
        language: &str,
        code: It,
    ) -> Vec<Result<Submission>> {
        let mut last = Vec::new();
        let mut result: Vec<Result<Submission>> = Vec::new();
        last.resize_with(self.session.len(), || None);
        for (index, code) in code.into_iter().enumerate() {
            let id = self.list.get().await;
            let account = &self.session[id];
            if let Some((index, r)) = take(&mut last[id]) {
                result[index] = get_result(r, account).await;
            }
            result.push(unsafe { MaybeUninit::uninit().assume_init() });
            match submit(account, problem, language, code.as_str()).await {
                Ok(_) => {
                    last[id] = {
                        let account_ptr: *const Session = account;
                        let problem_ptr: *const Problem = problem;
                        Some((
                            index,
                            spawn_local(unsafe {
                                get_last_submission(&*account_ptr, &*problem_ptr)
                            }),
                        ))
                    }
                }
                Err(e) => result[index] = Err(e),
            }
        }
        for (id, val) in last.into_iter().enumerate() {
            if let Some((index, r)) = val {
                result[index] = get_result(r, &self.session[id]).await;
            }
        }
        result
    }
}
