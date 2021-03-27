extern crate futures;
extern crate tokio;

use crate::{
    account::Account,
    config::submitter::{DELAY_PER_ACCOUNT, SUBMISSION_GET_DELAY, SUBMIT_DELAY},
    judge::{problem::Problem, session::Session, submission::Submission},
    types::Result,
};
use core::cmp::{Ord, PartialOrd};
use std::{
    cmp::{max, Reverse},
    collections::BinaryHeap,
    mem::take,
    vec::Vec,
};
use tokio::time::{sleep, sleep_until, Instant};

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
    pub async fn login(&mut self, mut accounts: Vec<Account>) -> Result<()> {
        self.list.expand(accounts.len());
        self.session.reserve(accounts.len());
        while !accounts.is_empty() {
            self.session
                .push(Session::from_account(accounts.pop().unwrap()).await?);
        }
        Ok(())
    }
    pub async fn submit(
        &mut self,
        problem: &Problem,
        language: &str,
        code: &str,
    ) -> Result<Submission> {
        let account = self.list.get().await;
        self.session[account]
            .submit(problem, language, code)
            .await?;
        sleep(SUBMISSION_GET_DELAY).await;
        self.session[account].get_last_submission(problem).await
    }
    pub async fn submit_vec(
        &mut self,
        problem: &Problem,
        language: &str,
        code: Vec<String>,
    ) -> Result<Vec<Submission>> {
        let mut last = Vec::new();
        let mut result = Vec::new();
        last.resize_with(self.session.len(), || None);
        for (index, code) in code.iter().enumerate() {
            let id = self.list.get().await;
            if let Some((index, r)) = take(&mut last[id]) {
                result[index] = r.await?;
            }
            self.session[id].submit(problem, language, code).await?;
            last[id] = Some((index, self.session[id].get_last_submission(problem)));
        }
        for (index, r) in last.into_iter().flatten() {
            result[index] = r.await?;
        }
        Ok(result)
    }
    pub fn is_empty(&self) -> bool {
        self.session.is_empty()
    }

    pub async fn logout(&mut self) -> Result<()> {
        for i in &self.session {
            i.logout().await?;
        }
        self.session.clear();
        self.list.clear();
        Ok(())
    }
}
impl Default for Submitter {
    fn default() -> Self {
        Self::new()
    }
}
