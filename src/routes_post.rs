use crate::deserializables::AuthAttempt;
use crate::deserializables::CreateAttempt;
use crate::functions::check_claims_csrf;
use crate::functions::handle_jwt_error;
use crate::requestguards::APIPath;
use crate::requestguards::AuthError;
use crate::requestguards::CSRFClaims;
use crate::responders::ApiResponse;
use crate::routes_get::api;
use crate::serializables::Claims;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::tokens::issue_token;
use rocket::http::Cookie;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::serde::json::Value;
use rocket::State;
use std::path::PathBuf;

// All routes mounted at api base Path

#[post("/?auth", format = "json", data = "<message>")]
pub(crate) fn auth_index<'a>(
    message: Json<AuthAttempt>,
    csrf: Result<CSRFClaims, AuthError>,
    cookies: &'a CookieJar,
    apikey: &State<ApiKey>,
    consts: &State<ZKConfig>,
) -> ApiResponse {
    auth("./".into(), message, csrf, cookies, apikey, consts)
}

#[post("/<path..>?auth", format = "json", data = "<message>")]
pub(crate) fn auth<'a>(
    path: PathBuf,
    message: Json<AuthAttempt>,
    csrf: Result<CSRFClaims, AuthError>,
    mut cookies: &'a CookieJar,
    apikey: &State<ApiKey>,
    consts: &State<ZKConfig>,
) -> ApiResponse {
    if csrf.is_err() {
        return handle_jwt_error(path, consts, apikey, &csrf.err().unwrap());
    }
    let absolutepath = PathBuf::from(consts.repo_files_location.clone() + &message.username);
    if !absolutepath.exists() || message.password != consts.admin_password {
        return handle_jwt_error(path, consts, apikey, &AuthError::WrongUsernamePassword);
    }
    let claims = Claims::default()
        .set_iss(consts.hostname.as_str())
        .set_sub(message.username.as_str())
        .set_aud(path.to_str().unwrap_or_default());
    cookies.add_private(Cookie::new(
        "jwt",
        issue_token(&claims, apikey.inner()).unwrap(),
    ));
    return api(APIPath(path), Ok(claims), consts, apikey);
}

#[allow(unused)] // TODO: Implement creation
#[post("/?new", format = "json", data = "<message>")]
pub(crate) fn create_index(
    csrf: Result<CSRFClaims, AuthError>,
    claims: Result<Claims, AuthError>,
    message: Json<CreateAttempt>,
    apikey: &State<ApiKey>,
    consts: &State<ZKConfig>,
) -> ApiResponse {
    create("./".into(), csrf, claims, message, apikey, consts)
}

#[allow(unused)] // TODO: Implement creation
#[post("/<path..>?new", format = "json", data = "<message>")]
pub(crate) fn create(
    path: PathBuf,
    csrf: Result<CSRFClaims, AuthError>,
    claims: Result<Claims, AuthError>,
    message: Json<CreateAttempt>,
    apikey: &State<ApiKey>,
    consts: &State<ZKConfig>,
) -> ApiResponse {
    if let Some(e) = check_claims_csrf(&claims, Some(&csrf)) {
        handle_jwt_error(path, consts, apikey, e)
    } else {
        ApiResponse::ok(ResponseBodyGeneric::default())
    }
}
