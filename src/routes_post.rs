use crate::deserializables::AuthAttempt;
use crate::functions::handle_jwt_error;
use crate::requestguards::AuthError;
use crate::requestguards::CSRFClaims;
use crate::responders::ApiResponse;
use crate::routes_get::api;
use crate::serializables::AppState;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::tokens::issue_token;
use crypto_hashes::sha2::{Digest, Sha256, Sha512};
use hex_literal::hex;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use std::path::Path;
use std::path::PathBuf;

// All routes mounted at api base Path

#[post("/", format = "json", data = "<message>")]
pub(crate) fn login_index(
    message: Json<AuthAttempt>,
    csrf: Result<CSRFClaims, AuthError>,
    mut cookies: Cookies,
    apikey: State<ApiKey>,
    consts: State<ZKConfig>,
) -> ApiResponse {
    login(PathBuf::from("./"), message, csrf, cookies, apikey, consts)
}

#[post("/<path..>", format = "json", data = "<message>")]
pub(crate) fn login(
    path: PathBuf,
    message: Json<AuthAttempt>,
    csrf: Result<CSRFClaims, AuthError>,
    mut cookies: Cookies,
    apikey: State<ApiKey>,
    consts: State<ZKConfig>,
) -> ApiResponse {
    if csrf.is_err() {
        return handle_jwt_error(path, consts, apikey, csrf.err().unwrap());
    }
    let absolutepath = PathBuf::from(consts.repo_files_location.clone() + &message.username);
    if !absolutepath.exists() || message.password != consts.admin_password {
        return handle_jwt_error(path, consts, apikey, AuthError::WrongUsernamePassword);
    }
    let claims = Claims::default()
        .set_iss(consts.hostname.as_str())
        .set_sub(message.username.as_str())
        .set_aud(path.to_str().unwrap_or_default());
    cookies.add_private(Cookie::new(
        "jwt",
        issue_token(&claims, apikey.inner()).unwrap(),
    ));
    return api(path, Ok(claims), consts, apikey);
}
