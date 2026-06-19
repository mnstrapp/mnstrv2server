use rand::prelude::*;
use sha2::{Digest, Sha512};
use std::fmt::Write;

#[allow(dead_code)]
pub fn hash_password(password: &str) -> String {
    let password_bytes = password.as_bytes();
    Sha512::digest(password_bytes)
        .iter()
        .fold(String::with_capacity(128), |mut hash, byte| {
            write!(&mut hash, "{byte:02x}").expect("writing to a String cannot fail");
            hash
        })
}

#[allow(dead_code)]
pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    hash_password(password) == hashed_password
}

pub fn generate_verification_code() -> String {
    let mut rng = rand::rng();
    let code = rng.random_range(10000..99999);
    code.to_string()
}
