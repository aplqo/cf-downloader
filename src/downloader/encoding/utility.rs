extern crate rand;
use rand::Rng;

pub(crate) fn random() -> u64 {
    rand::thread_rng().sample(rand::distributions::Standard)
}
