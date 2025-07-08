use rand::prelude::*;

pub fn generate_random_string() -> String {
    let rng = rand::rng();
    rng.sample_iter(&rand::distr::Alphanumeric)
        .take(32) 
        .map(char::from)
        .collect()
}