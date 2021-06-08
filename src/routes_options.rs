use rocket::http::Status;
use std::path::PathBuf;

#[options("/")]
pub(crate) fn options_mainpage() -> Status {
    Status::Ok
}

#[options("/<_param..>")]
pub(crate) fn options(_param: PathBuf) -> Status {
    Status::Ok
}
