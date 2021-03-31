extern crate tokio;

use super::{
    error::{Error, Kind, Operate, Result},
    Submitter,
};
use crate::{
    config::submitter::SUBMISSION_GET_DELAY,
    judge::{problem::Problem, submit::Submission, Result as JudgeResult, Session},
};
use std::mem::{take, MaybeUninit};
use tokio::{
    task::{spawn, JoinHandle},
    time::sleep,
};

async fn get_last_submission(session: &Session, problem: &Problem) -> Result<Submission> {
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
async fn submit(session: &Session, problem: &Problem, language: &str, code: &str) -> Result<()> {
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
    handle: JoinHandle<JudgeResult<Submission>>,
    session: &Session,
) -> Result<Submission> {
    match handle.await {
        Ok(v) => v,
        Err(e) => Error {
            operate: Operate::GetSubmission,
            kind: Kind::Join(e),
            handle: session.handle.clone(),
        },
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
        get_last_submission(account, problem)
    }

    pub async fn submit_iter<It: IntoIterator<Item = String>>(
        &mut self,
        problem: &Problem,
        language: &str,
        code: It,
    ) -> Vec<Result<Submission>> {
        let mut last = Vec::new();
        let mut result = Vec::new();
        last.resize(self.session.len(), None);
        result.resize(code.len(), unsafe { MaybeUninit::uninit().assume_init() });
        for (index, code) in code.into_iter().enumerate() {
            let id = self.list.get().await;
            let account = &self.session[id];
            if let Some((index, r)) = take(&mut last[id]) {
                result[index] = get_result(r, account).await;
            }
            match submit(account, problem, language, code).await {
                Ok(_) => last[id] = Some((index, spawn(get_last_submission(account, problem)))),
                Err(e) => result[index] = Err(e),
            }
        }
        for (id, val) in last.iter().enumerate() {
            if let Some((index, r)) = val {
                result[index] = get_result(r, &self.session[id]);
            }
        }
        result
    }
}
