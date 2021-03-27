pub mod retry {
    use std::time::Duration;
    pub const RETRY_COUNT: u32 = 10;
    pub const RETRY_DELAY: Duration = Duration::from_millis(200);
    pub const FORBIDDEN_DELAY: Duration = Duration::from_secs(120);
}
pub mod submission {
    use std::time::Duration;
    pub const CHECK_DELAY: Duration = Duration::from_secs(2);
}
pub mod submitter {
    use std::time::Duration;
    pub const DELAY_PER_ACCOUNT: Duration = Duration::from_secs(15);
    pub const SUBMISSION_GET_DELAY: Duration = Duration::from_secs(1);
    pub const SUBMIT_DELAY: Duration = Duration::from_secs(5);
}
