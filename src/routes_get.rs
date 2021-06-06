use crate::filesystem_interact::get_all_directory;
use crate::filesystem_interact::get_file;
use crate::filesystem_interact::DirectoryEntry;
use crate::filesystem_interact::FType;
use crate::git_interact::RepositoryTransaction;
use crate::requestguards::AuthError;
use crate::requestguards::CSRFClaims;
use crate::responders::ApiResponse;
use crate::serializables::AppState;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::functions::handle_jwt_error;
use rocket::State;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

// All Routes mounted at API base path
#[get("/", format = "json", rank = 2)]
pub(crate) fn api_index(
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    api(PathBuf::from("./"), claims, consts, key)
}

#[get("/<path..>", format = "json", rank = 1)]
pub(crate) fn api(
    path: PathBuf,
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    match claims {
        Ok(c) => handle_dir_file(path, c, consts, key),
        Err(e) => handle_jwt_error(path, consts, key, e),
    }
}

fn handle_dir_file(
    path: PathBuf,
    claims: Claims,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    let mut absolutepath = PathBuf::from(&consts.repo_files_location);
    absolutepath.push(claims.get_sub());
    absolutepath.push(&path);
    match &absolutepath {
        e if e.is_file() && e.extension() == Some(OsString::from("md").as_os_str()) => {
            handle_markdown_file(path, absolutepath, claims, consts, key)
        }
        e if e.is_dir() => handle_directory(path, absolutepath, claims, consts, key),
        _ => handle_invalid_path(path, claims, consts, key),
    }
}

fn handle_directory(
    apipath: PathBuf,
    absolutepath: PathBuf,
    claims: Claims,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(apipath.to_str().unwrap_or_default(), &key, &claims)
        .set_inner(
            json!(get_all_directory(get_file(absolutepath).unwrap(), false).ok()),
            DataType::Directory,
        )
        .set_history(true, apipath.to_str().unwrap_or_default())
        .set_appstate(AppState::default().set_authorized(true));
    ApiResponse::ok_json(res)
}

fn handle_markdown_file(
    apipath: PathBuf,
    absolutepath: PathBuf,
    claims: Claims,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(apipath.to_str().unwrap_or_default(), &key, &claims)
        .set_inner(json!(get_file(absolutepath).ok()), DataType::MD)
        .set_history(true, apipath.to_str().unwrap_or_default())
        .set_appstate(AppState::default().set_authorized(true));
    ApiResponse::ok_json(res)
}

fn handle_invalid_path(
    path: PathBuf,
    claims: Claims,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap_or_default(), &key, &claims)
        .set_inner(
            json!({"message": "Invalid file path."}),
            DataType::ErrorMessage,
        )
        .set_appstate(AppState::default().set_authorized(true));
    ApiResponse::not_found_json(res)
}