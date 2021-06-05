use crate::deserializables::AuthAttempt;
use crate::responders::ApiResponse;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::tokens::issue_token;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use std::path::Path;
use std::path::PathBuf;

// All routes mounted at api base Path

#[post("/", format = "json", data = "<message>")]
pub(crate) fn login_mainpage(
    message: Json<AuthAttempt>,
    mut cookies: Cookies,
    apikey: State<ApiKey>,
    consts: State<ZKConfig>,
) -> ApiResponse {
    let mut path = PathBuf::new();
    path.push(&consts.repo_files_location);
    path.push(&message.username);
    if message.password == consts.admin_password && path.exists() && path.is_dir() {
        cookies.add_private(Cookie::new(
            "jwt",
            issue_token(
                Claims::new(message.username.as_str(), &consts),
                apikey.inner(),
            )
            .unwrap(),
        ));
        let res = ResponseBodyGeneric::empty(
            "/",
            &apikey,
            &Claims::new(message.username.as_str(), &consts),
        )
        .inner(json!({"status": "ok"}), DataType::Ignore);
        ApiResponse::ok(res.json())
    } else {
        ApiResponse::unauthorized("Bad username/password.")
    }
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
        cookies.add_private(Cookie::new(
            "jwt",
            issue_token(
                Claims::new(message.username.as_str(), &consts),
                apikey.inner(),
            )
            .unwrap(),
        ));
        let res = ResponseBodyGeneric::empty(
            path.to_str().unwrap(),
            &apikey,
            &Claims::new(message.username.as_str(), &consts),
        )
        .inner(json!({"status": "ok"}), DataType::Ignore);
        ApiResponse::ok(res.json())
    } else {
        ApiResponse::unauthorized("Bad username/password.")
    }
}
