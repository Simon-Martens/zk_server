use crate::deserializables::AuthAttempt;
use crate::responders::ApiResponse;
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
    mut cookies: Cookies,
    apikey: State<ApiKey>,
    consts: State<ZKConfig>,
) -> ApiResponse {
    login(PathBuf::from("/"), message, cookies, apikey, consts)
}

#[post("/<path..>", format = "json", data = "<message>")]
pub(crate) fn login(
    path: PathBuf,
    message: Json<AuthAttempt>,
    mut cookies: Cookies,
    apikey: State<ApiKey>,
    consts: State<ZKConfig>,
) -> ApiResponse {
    if message.password == consts.admin_password {
        let claims = Claims::default()
            .set_iss(consts.hostname.as_str())
            .set_sub(message.username.as_str())
            .set_aud("/");
        cookies.add_private(Cookie::new(
            "jwt",
            issue_token(&claims, apikey.inner()).unwrap(),
        ));
        let res = ResponseBodyGeneric::default()
            .set_apiurl(path.to_str().unwrap(), &apikey, &claims)
            .set_inner(json!({"status": "ok"}), DataType::Ignore);
        ApiResponse::ok_json(res.json())
    } else {
        ApiResponse::unauthorized_message("Bad username/password.")
    }
}
