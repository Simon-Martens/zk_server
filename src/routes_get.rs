use crate::filesystem_interact::ls;
use crate::filesystem_interact::open;
use crate::filesystem_interact::Entry;
use crate::filesystem_interact::FType;
use crate::functions::check_claims_csrf;
use crate::functions::handle_jwt_error;
use crate::requestguards::AuthError;
use crate::responders::ApiResponse;
use crate::serializables::AppState;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use rocket::State;
use std::path::PathBuf;

// All Routes mounted at API base path
#[get("/", format = "json", rank = 2)]
pub(crate) fn api_index(
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    api("./".into(), claims, consts, key)
}

#[get("/<path..>", format = "json", rank = 1)]
pub(crate) fn api(
    path: PathBuf,
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    if let Some(e) = check_claims_csrf(&claims, None) {
        handle_jwt_error(path, consts, key, e)
    } else {
        handle_dir_file(path, claims.unwrap(), consts, key)
    }
}

fn handle_dir_file(
    path: PathBuf,
    claims: Claims,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    let mut basepath = PathBuf::from(&consts.repo_files_location);
    basepath.push(claims.get_sub());
    if let Some(e) = open(&path, &basepath) {
        match e.ftype {
            FType::MDFile => handle_markdown_file(path, e, claims, key),
            FType::Directory => handle_directory(path, e, claims, key, basepath),
        }
    } else {
        handle_invalid_path(path, claims, key)
    }
}

fn handle_directory(
    path: PathBuf,
    dir: Entry,
    claims: Claims,
    key: State<ApiKey>,
    basepath: PathBuf,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap_or_default(), &key, &claims)
        .set_inner(
            ls(dir, &basepath, false, "").map_or(json!(""), |c| c.json()),
            DataType::Directory,
        )
        .set_history(true, path.to_str().unwrap_or_default())
        .set_appstate(AppState::default().set_authorized(true));
    ApiResponse::ok(res)
}

fn handle_markdown_file(
    path: PathBuf,
    mdfile: Entry,
    claims: Claims,
    key: State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap_or_default(), &key, &claims)
        .set_inner(mdfile.json(), DataType::MD)
        .set_history(true, path.to_str().unwrap_or_default())
        .set_appstate(AppState::default().set_authorized(true));
    ApiResponse::ok(res)
}

fn handle_invalid_path(
    path: PathBuf,
    claims: Claims,
    key: State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap_or_default(), &key, &claims)
        .set_inner(
            json!({"message": "Invalid file path."}),
            DataType::ErrorMessage,
        )
        .set_appstate(AppState::default().set_authorized(true));
    ApiResponse::not_found(res)
}
