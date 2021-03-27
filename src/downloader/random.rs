extern crate rand;

use rand::{
    distributions::{Alphanumeric, Standard},
    prelude::Distribution,
    thread_rng, Rng,
};
use std::iter;

pub fn random_string(length: usize) -> String {
    iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(length)
        .collect()
}
pub fn random_standard<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().sample(Standard)
}
