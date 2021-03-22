extern crate reqwest;
extern crate tokio;

use crate::types::Result;
use std::{result::Result as StdResult, time::Duration};
use tokio::time::sleep;
include!("../config/retry.rs");

pub(super) async fn async_retry<'a, F, U, Out>(fun: F) -> Result<Out>
where
    F: Fn() -> U,
    U: core::future::Future<Output = StdResult<Out, reqwest::Error>>,
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
