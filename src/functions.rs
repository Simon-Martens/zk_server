use crate::requestguards::AuthError;
use crate::requestguards::CSRFClaims;
use crate::responders::ApiResponse;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use rocket::State;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;

// Helpers
#[allow(dead_code)] // Will be needed eventually... Implement it using SHA-2 for file names
fn calculate_id<T: Hash>(t: &T) -> u64 {
    let salt: u64 = rand::random();
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.write_u64(salt);
    s.finish()
}

pub(crate) fn check_claims_csrf<'a>(
    claims: &'a Result<Claims, AuthError>,
    csrf: Option<&'a Result<CSRFClaims, AuthError>>,
) -> Option<&'a AuthError> {
    if claims.is_err() {
        claims.as_ref().err().into()
    } else if csrf.is_some() && csrf.unwrap().is_err() {
        csrf.unwrap().as_ref().err().into()
    } else {
        None
    }
}

pub(crate) fn handle_jwt_error(
    path: PathBuf,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
    error: &AuthError,
) -> ApiResponse {
    // TODO MATCH MESSAGE TO AUTH ERROR
    let res = ResponseBodyGeneric::default().set_apiurl(
        path.to_str().unwrap_or_default(),
        &key,
        &Claims::default().set_iss(consts.hostname.as_str()),
    );
    let res = match error {
        AuthError::UsernameInvalidated => res.set_inner(
            json!({"message": "Username invalidated."}),
            DataType::ErrorMessage,
        ),
        AuthError::Missing => res.set_inner(
            json!({"message": "JWT missing. Please authorize."}),
            DataType::ErrorMessage,
        ),
        AuthError::WrongUsernamePassword => res.set_inner(
            json!({"message": "Bad username/password."}),
            DataType::ErrorMessage,
        ),
        AuthError::CSRFError(_) => res.set_inner(
            json!({"message": "Bad or invalid CSRF-Token."}),
            DataType::ErrorMessage,
        ),
        _ => res.set_inner(
            json!({"message": "JWT invalidated. Please authorize."}),
            DataType::ErrorMessage,
        ),
    };
    ApiResponse::unauthorized(res)
}
