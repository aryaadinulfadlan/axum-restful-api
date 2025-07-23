use rand::prelude::*;

pub fn generate_random_string(n: u8) -> String {
    let rng = rand::rng();
    rng.sample_iter(&rand::distr::Alphanumeric)
        .take(n as usize)
        .map(char::from)
        .collect()
}