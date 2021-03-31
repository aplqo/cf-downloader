extern crate tokio;

use crate::{
    account::Account,
    config::submitter::{DELAY_PER_ACCOUNT, SUBMISSION_GET_DELAY, SUBMIT_DELAY},
    judge::{self, problem::Problem, session::Session, submit::Submission},
};
use core::cmp::{Ord, PartialOrd};
use futures::future::join_all;
use std::{
    cmp::{max, Reverse},
    collections::BinaryHeap,
    error::Error as StdError,
    fmt,
    mem::{take, MaybeUninit},
    result::Result as StdResult,
    vec::Vec,
};
use tokio::{
    task::{spawn, JoinError, JoinHandle},
    time::{sleep, sleep_until, Instant},
};

#[derive(Debug, Clone, Copy)]
enum Operate {
    BuildClient,
    Login,
    Submit,
    GetSubmission,
    Logout,
}
#[derive(Debug)]
enum Kind {
    Join(JoinError),
    Judge(judge::error::Error),
}
#[derive(Debug)]
pub struct Error {
    operate: Operate,
    kind: Kind,
    handle: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Kind::Join(x) => write!(f, "Error joining task using {}: {}", self.handle, x),
            Kind::Judge(handle, x) => {
                write!(
                    f,
                    "Error while {} using {}: {}",
                    self.operate, self.handle, x
                )
            }
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Kind::Join(x) => Some(x),
            Kind::Judge(x) => Some(x),
        }
    }
}
impl fmt::Display for Operate {
    fn fmt(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildClient => f.write_str("building client"),
            Self::Login => f.write_str("login"),
            Self::Submit => f.write_str("submitting code"),
            Self::GetSubmission => f.write_str("getting submission"),
            Self::Logout => f.write_str("logout"),
        }
    }
}
type Result<T> = StdResult<T, Error>;

#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct AccountNode {
    next_submit: Instant,
    id: usize,
}
struct AccountList {
    heap: BinaryHeap<Reverse<AccountNode>>,
    next_submit: Instant,
}
impl AccountList {
    fn expand(&mut self, count: usize) {
        let base = self.heap.len();
        let now = Instant::now();
        self.heap.reserve(count);
        for id in base..base + count {
            self.heap.push(Reverse(AccountNode {
                next_submit: now,
                id,
            }));
        }
    }
    async fn get(&mut self) -> usize {
        let account = self.heap.pop().unwrap().0;
        sleep_until(max(account.next_submit, self.next_submit)).await;
        self.next_submit += SUBMIT_DELAY;
        self.heap.push(Reverse(AccountNode {
            next_submit: account.next_submit + DELAY_PER_ACCOUNT,
            id: account.id,
        }));
        account.id
    }
    fn clear(&mut self) {
        self.heap.clear();
    }
}

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
    handle: JoinHandle<judge::error::Result<Submission>>,
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

pub struct Submitter {
    session: Vec<Session>,
    list: AccountList,
}
impl Submitter {
    pub fn new() -> Self {
        Submitter {
            session: Vec::new(),
            list: AccountList {
                heap: BinaryHeap::new(),
                next_submit: Instant::now(),
            },
        }
    }
    pub async fn login(&mut self, accounts: Vec<Account>) -> Vec<Error> {
        let old_size = self.session.len();
        self.session.reserve(accounts.len());
        let mut err = Vec::new();
        join_all(accounts.into_iter().map(async move |x| {
            let mut p = Session::with_proxy(x.proxy).map_err(|e| Error {
                operate: Operate::BuildClient,
                kind: Kind::Judge(e),
                handle: x.handle,
            })?;
            match p.login(x.handle, x.password.as_str()).await {
                Ok(_) => Ok(p),
                Err(e) => Err(Error {
                    operate: Operate::Login,
                    kind: Kind::Judge(e),
                    handle: p.handle,
                }),
            }
        }))
        .await
        .for_each(|x| match x {
            Ok(v) => self.session.push(v),
            Err(e) => err.push(e),
        });
        self.list.expand(self.session.len() - old_size);
        return err;
    }
    pub async fn add_session(&mut self, mut sessions: Vec<Session>) {
        self.list.expand(sessions.len());
        self.session.append(&mut sessions);
    }
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
    pub async fn submit_vec(
        &mut self,
        problem: &Problem,
        language: &str,
        code: Vec<String>,
    ) -> Vec<Result<Submission>> {
        let mut last = Vec::new();
        let mut result = Vec::new();
        last.resize(self.session.len(), None);
        result.resize(code.len(), unsafe { MaybeUninit::uninit().assume_init() });
        for (index, code) in code.iter().enumerate() {
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
    pub fn is_empty(&self) -> bool {
        self.session.is_empty()
    }

    pub async fn logout(&mut self) -> Vec<Error> {
        let ret = join_all(self.session.iter_mut().filter_map(
            async move |x| match x.logout().await {
                Ok(_) => None,
                Err(e) => Some(Error {
                    operate: Operate::Logout,
                    kind: Kind::Judge(e),
                    handle: x.handle.clone(),
                }),
            },
        ))
        .await;
        if ret.is_empty() {
            self.session.clear();
            self.list.clear();
        }
        ret
    }
}
impl Default for Submitter {
    fn default() -> Self {
        Self::new()
    }
}
