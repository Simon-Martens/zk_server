use std::path::PathBuf;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use crate::state::ApiKey;
use crate::state::Consts;
use crate::responders::ApiResponse;
use crate::deserializables::AuthAttempt;
use crate::serializables::Claims;
use crate::tokens::issue_token;
use crate::serializables::ResponseBodyGeneric;

#[post("/api", format="json", data="<message>")] 
pub(crate) fn login_mainpage(message: Json<AuthAttempt>, mut cookies: Cookies, apikey: State<ApiKey>, consts: State<Consts>) -> ApiResponse {
    if message.password == consts.password {
        cookies.add_private(Cookie::new(
            "jwt", 
            issue_token(Claims::new(message.username.clone(), consts), apikey.inner()).unwrap())
        );
        ApiResponse {
            status: Status::Ok,
            json: ResponseBodyGeneric::lazy(json!({"status": "ok"}))
        }
    } else {
        ApiResponse {
            status: Status::Forbidden,
            json: ResponseBodyGeneric::lazy(json!({"status": "bad username/password"}))
        }
    }
}

#[post("/api/<path..>", format="json", data="<message>")]
pub(crate) fn login(path: PathBuf, message: Json<AuthAttempt>, mut cookies: Cookies, apikey: State<ApiKey>, consts: State<Consts>) -> ApiResponse {
    if message.password == consts.password {
        cookies.add_private(Cookie::new(
            "jwt", 
            issue_token(Claims::new(message.username.clone(), consts), apikey.inner()).unwrap()));
        ApiResponse {
            status: Status::Ok,
            json: ResponseBodyGeneric::lazy(json!({"status": "ok"}))
        }
    } else {
        ApiResponse {
            status: Status::Forbidden,
            json: ResponseBodyGeneric::lazy(json!({"status": "bad username/password"}))
        }
    }
}