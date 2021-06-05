use crate::requestguards::AuthError;
use crate::responders::ApiResponse;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use rocket::State;
use std::path::Path;
use std::path::PathBuf;

// All Routes mounted at API base path

#[get("/<path..>", format = "json", rank = 1)]
pub(crate) fn api(
    path: PathBuf,
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    match claims {
        Ok(c) => {
            let res = ResponseBodyGeneric::empty(path.to_str().unwrap(), &key, &c)
                .inner(json!({"status": "ok"}), DataType::Ignore);
            ApiResponse::ok(res.json())
        }
        Err(e) => handle_jwt_error(e),
    }
}

#[get("/", format = "json", rank = 2)]
pub(crate) fn api_index(
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    match claims {
        Ok(c) => {
            let res = ResponseBodyGeneric::empty("/", &key, &c)
                .inner(json!({"status": "ok"}), DataType::Ignore);
            ApiResponse::ok(res.json())
        }
        Err(e) => handle_jwt_error(e),
    }
}

fn handle_jwt_error(error: AuthError) -> ApiResponse {
    // TODO MATCH MESSAGE TO AUTH ERROR
    match error {
        AuthError::UsernameInvalidated => ApiResponse::forbidden("Username invalidated."),
        AuthError::Missing => ApiResponse::unauthorized("JWT missing. Please authorize."),
        AuthError::JWTError(n) => match n {
            _ => ApiResponse::unauthorized("Authorization failure."),
        },
    }
}
