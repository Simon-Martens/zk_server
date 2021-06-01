use rocket::request::Request;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::State;
use rocket::http::Status;
use jsonwebtoken::errors::Error;
use crate::state::ApiKey;
use crate::state::Consts;
use crate::tokens::validate_token;
use crate::serializables::Claims;

// TODO: Error response
#[derive(Debug)]
pub(crate) enum AuthError {
    Missing,
    JWTError(Error)
}

impl<'a, 'r> FromRequest<'a, 'r> for Claims {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Option<String> =request.cookies()
            .get_private("jwt")
            .and_then(|cookie| cookie.value().parse().ok());
        if keys == None {
            return Outcome::Failure((Status::Unauthorized, AuthError::Missing));
        }
        let apikey = request.guard::<State<ApiKey>>();
        let consts = request.guard::<State<Consts>>();
        let validation = validate_token(&keys.unwrap(), &apikey.unwrap(), consts.unwrap());
        match validation {
            Err(e) => Outcome::Failure((Status::Unauthorized, AuthError::JWTError(e))),
            Ok(n) => Outcome::Success(n.claims)
        }
    }
}