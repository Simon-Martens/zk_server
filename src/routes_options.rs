use rocket::http::Status;
use std::path::PathBuf;

#[options("/")]
pub(crate) async fn options_mainpage() -> Status {
    Status::Ok
}

#[options("/<_param..>")]
pub(crate) async fn options(_param: PathBuf) -> Status {
    Status::Ok
}
