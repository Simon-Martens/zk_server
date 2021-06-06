use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;

use rocket::State;

use crate::requestguards::AuthError;
use crate::responders::ApiResponse;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;

// Helpers
fn calculate_id<T: Hash>(t: &T) -> u64 {
    let salt: u64 = rand::random();
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.write_u64(salt);
    s.finish()
}

pub(crate) fn handle_jwt_error(
    path: PathBuf,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
    error: AuthError,
) -> ApiResponse {
    // TODO MATCH MESSAGE TO AUTH ERROR
    let mut res = ResponseBodyGeneric::default().set_apiurl(
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
    ApiResponse::unauthorized_message(res)
}
