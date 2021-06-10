use crate::serializables::Claims;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::tokens::validate_token;
use jsonwebtoken::errors::Error;
use rocket::http::uri::error::PathError;
use rocket::http::uri::fmt::Path;
use rocket::http::uri::Segments;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::request::FromSegments;
use rocket::request::Outcome;
use rocket::request::Request;
use rocket::State;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum AuthError {
    Missing,
    WrongUsernamePassword,
    UsernameInvalidated,
    PathTraversalAttempt,
    CSRFError(Error),
    JWTError(Error),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Option<String> = request
            .cookies()
            .get_private("jwt")
            .and_then(|cookie| cookie.value().parse().ok());
        if keys == None {
            return Outcome::Failure((Status::Unauthorized, AuthError::Missing));
        }
        let apikey = request.guard::<&State<ApiKey>>().await;
        let consts = request.guard::<&State<ZKConfig>>().await;
        let validation = validate_token(&keys.unwrap(), &apikey.unwrap(), &consts.unwrap());
        match validation {
            Err(e) => Outcome::Failure((Status::Unauthorized, AuthError::JWTError(e))),
            Ok(n) => {
                if n.claims.get_sub().is_empty() || PathBuf::from(n.claims.get_sub()).is_absolute()
                {
                    return Outcome::Failure((Status::Forbidden, AuthError::PathTraversalAttempt));
                }
                let mut path = PathBuf::from(consts.unwrap().repo_files_location.clone());
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CSRFClaims {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Option<&str> = request.headers().get_one("XSRF-TOKEN");
        if keys == None {
            return Outcome::Failure((Status::Unauthorized, AuthError::Missing));
        }
        let apikey = request.guard::<&State<ApiKey>>().await;
        let consts = request.guard::<&State<ZKConfig>>().await;
        let validation = validate_token(
            &keys.unwrap().to_string(),
            &apikey.unwrap(),
            &consts.unwrap(),
        );
        match validation {
            Err(e) => Outcome::Failure((Status::Unauthorized, AuthError::CSRFError(e))),
            Ok(n) => {
                // TODO: Check for matching route Path
                Outcome::Success(CSRFClaims(n.claims))
            }
        }
    }
}

pub(crate) struct APIPath(pub(crate) PathBuf);

impl<'r> FromSegments<'r> for APIPath {
    type Error = PathError;
    fn from_segments(segments: Segments<'r, Path>) -> Result<Self, Self::Error> {
        let ret: PathBuf = segments.to_path_buf(true)?;
        Ok(APIPath(ret))
    }
}
