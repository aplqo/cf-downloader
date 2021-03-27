extern crate reqwest;
extern crate tokio;

use crate::{
    config::retry::{FORBIDDEN_DELAY, RETRY_COUNT, RETRY_DELAY},
    types::Result,
};
use std::future::Future;
use tokio::time::sleep;

pub(super) async fn async_retry<'a, F, U, Out>(fun: F) -> Result<Out>
where
    F: Fn() -> U,
    U: Future<Output = reqwest::Result<Out>>,
{
    for _i in 0..RETRY_COUNT - 1 {
        match fun().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
                    sleep(FORBIDDEN_DELAY).await;
                } else {
                    sleep(RETRY_DELAY).await;
                }
            }
        }
    }
    match fun().await {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e)),
    }
}
