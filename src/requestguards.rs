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
    WrongUsernamePassword,
    UsernameInvalidated,
    PathTraversalAttempt,
    CSRFError(Error),
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
                if n.claims.get_sub().is_empty() || PathBuf::from(n.claims.get_sub()).is_absolute() {
                    return Outcome::Failure((Status::Forbidden, AuthError::PathTraversalAttempt));
                }
                let cfg = request.guard::<State<ZKConfig>>().unwrap();
                let mut path = PathBuf::from(&cfg.repo_files_location);
                path.push(n.claims.get_sub());
                if !path.exists() {
                    return Outcome::Failure((Status::Forbidden, AuthError::UsernameInvalidated));
                } 
                Outcome::Success(n.claims)
            }
        }
    }
}

pub(crate) struct CSRFClaims(Claims);

impl<'a, 'r> FromRequest<'a, 'r> for CSRFClaims {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Option<&str> = request
            .headers()
            .get_one("XSRF-TOKEN");
        if keys == None {
            return Outcome::Failure((Status::Unauthorized, AuthError::Missing));
        }
        let apikey = request.guard::<State<ApiKey>>();
        let consts = request.guard::<State<ZKConfig>>();
        let validation = validate_token(&keys.unwrap().to_string(), &apikey.unwrap(), &consts.unwrap());
        match validation {
            Err(e) => Outcome::Failure((Status::Unauthorized, AuthError::CSRFError(e))),
            Ok(n) => {
                // TODO: Check for matching route Path
                Outcome::Success(CSRFClaims(n.claims))
            }
        }
    }
}