use crate::serializables::Claims;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::tokens::validate_token;
use jsonwebtoken::errors::Error;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::request::Request;
use rocket::State;
use std::path::PathBuf;

// TODO: Error response
#[derive(Debug)]
pub(crate) enum AuthError {
    Missing,
    UsernameInvalidated,
    JWTError(Error),
}

impl<'a, 'r> FromRequest<'a, 'r> for Claims {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Option<String> = request
            .cookies()
            .get_private("jwt")
            .and_then(|cookie| cookie.value().parse().ok());
        if keys == None {
            return Outcome::Failure((Status::Unauthorized, AuthError::Missing));
        }
        let apikey = request.guard::<State<ApiKey>>();
        let consts = request.guard::<State<ZKConfig>>();
        let validation = validate_token(&keys.unwrap(), &apikey.unwrap(), &consts.unwrap());
        match validation {
            Err(e) => Outcome::Failure((Status::Unauthorized, AuthError::JWTError(e))),
            Ok(n) => {
                let cfg = request.guard::<State<ZKConfig>>().unwrap();
                let mut path = PathBuf::from(&cfg.repo_files_location);
                path.push(n.claims.get_sub());
                if !path.exists() {
                    Outcome::Failure((Status::Forbidden, AuthError::UsernameInvalidated))
                } else {
                    Outcome::Success(n.claims)
                }
            }
        }
    }
}
