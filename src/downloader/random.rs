extern crate rand;

use rand::{distributions::Standard, prelude::Distribution, thread_rng, Rng};

pub fn random_standard<T>() -> T
where
    Standard: Distribution<T>,
{
    thread_rng().sample(Standard)
}
pub fn random_hex(length: usize) -> String {
    let mut ret = thread_rng()
        .sample_iter::<u64, _>(Standard)
        .take(length + 15 / 16)
        .fold(String::new(), |mut acc, v| {
            acc.push_str(v.to_string().as_str());
            acc
        });
    ret.truncate(length);
    ret
}
