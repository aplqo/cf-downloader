pub mod retry {
    use std::time::Duration;
    pub const RETRY_COUNT: u32 = 10;
    pub const RETRY_DELAY: Duration = Duration::from_millis(200);
    pub const FORBIDDEN_DELAY: Duration = Duration::from_secs(120);
}
pub mod judge {
    pub mod session {
        pub const VERBOSE: bool = false;
        pub const BFAA: &str = "b182688b66909d3192211b04acf4ae61";
    }
    pub mod submit {
        use std::time::Duration;
        pub const CHECK_DELAY: Duration = Duration::from_secs(2);
    }
}
pub mod submitter {
    use std::time::Duration;
    pub const DELAY_PER_ACCOUNT: Duration = Duration::from_secs(15);
    pub const SUBMISSION_GET_DELAY: Duration = Duration::from_secs(1);
    pub const SUBMIT_DELAY: Duration = Duration::from_secs(5);
}
pub mod register {
    use std::time::Duration;
    pub const HANDLE_LEN: usize = 10;
    pub const PASSWORD_LEN: usize = 10;
    pub const REGISTER_DELAY: Duration = Duration::from_secs(5);
}
pub mod email {
    use std::time::Duration;
    pub const CHECK_DELAY: Duration = Duration::from_secs(5);
}
