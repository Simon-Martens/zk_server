use crate::state::ZKConfig;
use rocket::State;
use std::path::Path;
use std::path::PathBuf;

// All routes mounted at "/" base path. Used for static files serving only.
#[get("/", rank = 100)]
pub(crate) fn app(consts: State<ZKConfig>) -> Option<rocket::response::NamedFile> {
    if consts.cors || consts.static_files_location == None {
        return None;
    }
    let sf = consts.static_files_location.as_ref();
    rocket::response::NamedFile::open(Path::new(sf.unwrap()).join("index.html")).ok()
}

#[get("/<file..>", rank = 101)]
pub(crate) fn static_or_app(
    file: PathBuf,
    consts: State<ZKConfig>,
) -> Option<rocket::response::NamedFile> {
    if consts.cors || consts.static_files_location == None {
        return None;
    }
    let sf = consts.static_files_location.as_ref();
    match rocket::response::NamedFile::open(Path::new(sf.unwrap()).join(file)) {
        Ok(n) => Some(n),
        Err(_) => app(consts),
    }
}
