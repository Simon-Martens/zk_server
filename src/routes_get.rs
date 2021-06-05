use crate::git_interact::RepositoryTransaction;
use crate::requestguards::AuthError;
use crate::requestguards::CSRFClaims;
use crate::responders::ApiResponse;
use crate::serializables::Claims;
use crate::serializables::DataType;
use crate::serializables::ResponseBodyGeneric;
use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::filesystem_interact::get_directory_contents;
use crate::filesystem_interact::DirectoryEntry;
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
    api(PathBuf::from("/"), claims, consts, key)
}

#[get("/<path..>", format = "json", rank = 1)]
pub(crate) fn api(
    path: PathBuf,
    claims: Result<Claims, AuthError>,
    consts: State<ZKConfig>,
    key: State<ApiKey>,
) -> ApiResponse {
    match claims {
        Ok(c) => handle_dir_file(&path, &c, &consts, &key),
        Err(e) => handle_jwt_error(e),
    }
}

fn handle_dir_file(
    path: &PathBuf,
    claims: &Claims,
    consts: &State<ZKConfig>,
    key: &State<ApiKey>,
) -> ApiResponse {
    let mut p = PathBuf::from(&consts.repo_files_location);
    p.push(claims.get_sub());
    p.push(&path);
    match &p {
        e if p.is_file() && p.extension() == Some(OsString::from("md").as_os_str()) => {
            handle_markdown_file(&p, &claims, &consts, &key)
        }
        e if p.is_dir() => handle_directory(&p, &claims, &consts, &key),
        _ => handle_invalid_path(&p, &claims, &consts, &key),
    }
}

fn handle_directory(
    path: &PathBuf,
    claims: &Claims,
    consts: &State<ZKConfig>,
    key: &State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap(), &key, &claims)
        .set_inner(json!(get_directory_contents(&path, false).ok()), DataType::Directory);
    ApiResponse::ok_json(res.json())
}

fn handle_markdown_file(
    path: &PathBuf,
    claims: &Claims,
    consts: &State<ZKConfig>,
    key: &State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap(), &key, &claims)
        .set_inner(json!({"message": "markdownfilefound"}), DataType::MD);
    ApiResponse::ok_json(res.json())
}

fn handle_invalid_path(
    path: &PathBuf,
    claims: &Claims,
    consts: &State<ZKConfig>,
    key: &State<ApiKey>,
) -> ApiResponse {
    let res = ResponseBodyGeneric::default()
        .set_apiurl(path.to_str().unwrap(), &key, &claims)
        .set_inner(
            json!({"message": "Invalid file path."}),
            DataType::ErrorMessage,
        );
    ApiResponse::not_found_json(res.json())
}

fn handle_jwt_error(error: AuthError) -> ApiResponse {
    // TODO MATCH MESSAGE TO AUTH ERROR
    match error {
        AuthError::UsernameInvalidated => ApiResponse::forbidden_message("Username invalidated."),
        AuthError::Missing => ApiResponse::unauthorized_message("JWT missing. Please authorize."),
        AuthError::JWTError(n) => match n {
            _ => ApiResponse::unauthorized_message("Authorization failure."),
        },
    }
}
