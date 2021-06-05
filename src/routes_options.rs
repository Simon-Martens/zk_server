use rocket::http::Status;
use std::path::PathBuf;

#[options("/")]
pub(crate) fn options_mainpage() -> Status {
    Status::Ok
}

#[options("/<path..>")]
pub(crate) fn options(path: PathBuf) -> Status {
    Status::Ok
}
