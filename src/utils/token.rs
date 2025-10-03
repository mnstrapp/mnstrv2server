use rocket::{
    Request,
    request::{FromParam, FromRequest, Outcome},
};
use serde::{Deserialize, Serialize};

/// Represents an unverified raw token as received from the client
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RawToken {
    /// The raw token string
    pub value: String,
}

/// Implements Rocket's FromRequest trait to extract the token from the Authorization header
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RawToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let token = request
            .headers()
            .get_one("Authorization")
            .map(|header| header.split(" ").nth(1).unwrap_or(""));
        Outcome::Success(
            request
                .local_cache(|| RawToken {
                    value: token.unwrap_or("").to_string(),
                })
                .clone(),
        )
    }
}

/// Implements Rocket's FromParam trait to extract the token from path parameters
impl<'r> FromParam<'r> for RawToken {
    type Error = ();

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(RawToken {
            value: param.to_string(),
        })
    }
}
