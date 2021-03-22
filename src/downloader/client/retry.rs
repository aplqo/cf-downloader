use crate::types::Result;
use std::{result::Result as StdResult, time::Duration};
use tokio::time::sleep;
include!("../config/retry.rs");

pub(super) async fn async_retry<'a, F, U, E: 'static, Out>(fun: F) -> Result<Out>
where
    F: Fn() -> U,
    E: std::error::Error,
    U: core::future::Future<Output = StdResult<Out, E>>,
{
    for _i in 0..RETRY_COUNT - 1 {
        if let Ok(v) = fun().await {
            return Ok(v);
        }
        sleep(RETRY_DELAY).await;
    }
    match fun().await {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e)),
    }
}
