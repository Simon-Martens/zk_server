use std::path::PathBuf;
use std::path::Path;
use rocket::State;
use rocket::http::Status;
use crate::serializables::Claims;
use crate::serializables::ResponseBodyGeneric;
use crate::requestguards::AuthError;
use crate::responders::ApiResponse;
use crate::state::Consts;

// GET STATIC FILES
#[get("/", rank = 3)]
pub(crate) fn app(consts: State<Consts>) -> Option<rocket::response::NamedFile> {
    rocket::response::NamedFile::open(
        Path::new(consts.static_file_location).join("index.html")
    ).ok()
}

#[get("/<file..>", rank = 4)]
pub(crate) fn static_or_app(file: PathBuf, consts: State<Consts>) -> Option<rocket::response::NamedFile> {
    match rocket::response::NamedFile::open(
        Path::new(consts.static_file_location).join(file)
    ) {
        Ok(n) => Some(n),
        Err(_) => app(consts)
    }
}

// GET API CALLS
#[get("/api/<path..>", format="json")]
pub(crate) fn api(path: PathBuf, claims: Result<Claims, AuthError>, consts: State<Consts>) -> ApiResponse {
    let mut p = PathBuf::new();
    p.push(consts.repo_file_location);
    p.push(path);
    match claims {
        Ok(_) => ApiResponse {
            status: Status::Ok,
            json: ResponseBodyGeneric::lazy(json!({"status": "ok", "path": p.to_str(), "absolute": p.is_absolute(), "isfile": p.is_file(), "isdir": p.is_dir()})),
        },
        Err(e) => handle_jwt_error(e)
    }
}

#[get("/api", format="json")]
pub(crate) fn api_index(claims: Result<Claims, AuthError>, consts: State<Consts>) -> ApiResponse {
    match claims {
        Ok(_) => ApiResponse {
            status: Status::Ok,
            json: ResponseBodyGeneric::lazy(json!({"status": "ok"})),
        },
        Err(e) => handle_jwt_error(e)
    }
}

fn handle_jwt_error(error: AuthError) -> ApiResponse {
    ApiResponse {
        status: Status::Unauthorized,
        json: ResponseBodyGeneric::lazy(json!({"status": "unauthorized"}))
    }
}