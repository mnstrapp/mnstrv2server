use sha2::{Digest, Sha512};

#[allow(dead_code)]
pub fn hash_password(password: &str) -> String {
    format!("{:x}", Sha512::digest(password.as_bytes()))
}

#[allow(dead_code)]
pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    hash_password(password) == hashed_password
}
