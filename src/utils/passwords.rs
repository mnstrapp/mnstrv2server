use rand::Rng;
use sha2::{Digest, Sha512};

#[allow(dead_code)]
pub fn hash_password(password: &str) -> String {
    format!("{:x}", Sha512::digest(password.as_bytes()))
}

#[allow(dead_code)]
pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    hash_password(password) == hashed_password
}

pub fn generate_verification_code() -> String {
    let code = rand::rng().random_range(10000..99999);
    code.to_string()
}
