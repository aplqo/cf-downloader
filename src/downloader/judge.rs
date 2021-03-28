pub mod problem;
pub mod register;
mod retry;
mod search;
pub mod session;
pub mod submit;
pub(crate) mod verdict;

struct UtilityRegex {
    session: session::RegexSet,
    submit: submit::RegexSet,
}
impl UtilityRegex {
    fn new() -> Self {
        Self {
            session: session::RegexSet::new(),
            submit: submit::RegexSet::new(),
        }
    }
}
