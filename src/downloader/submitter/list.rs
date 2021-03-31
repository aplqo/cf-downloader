extern crate tokio;

use crate::config::submitter::{DELAY_PER_ACCOUNT, SUBMIT_DELAY};
use std::{
    cmp::{max, Reverse},
    collections::BinaryHeap,
};
use tokio::time::{sleep_until, Instant};

#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct AccountNode {
    next_submit: Instant,
    id: usize,
}
pub(super) struct AccountList {
    heap: BinaryHeap<Reverse<AccountNode>>,
    next_submit: Instant,
}

impl AccountList {
    pub(super) fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            next_submit: Instant::now(),
        }
    }
    pub(super) fn expand(&mut self, count: usize) {
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
    pub(super) async fn get(&mut self) -> usize {
        let account = self.heap.pop().unwrap().0;
        sleep_until(max(account.next_submit, self.next_submit)).await;
        self.next_submit += SUBMIT_DELAY;
        self.heap.push(Reverse(AccountNode {
            next_submit: account.next_submit + DELAY_PER_ACCOUNT,
            id: account.id,
        }));
        account.id
    }
    pub(super) fn clear(&mut self) {
        self.heap.clear();
    }
}
