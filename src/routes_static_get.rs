use crate::state::ZKConfig;
use rocket::fs::NamedFile;
use rocket::State;
use std::path::Path;
use std::path::PathBuf;

// All routes mounted at "/" base path. Used for static files serving only.
#[get("/", rank = 100)]
pub(crate) async fn app(consts: &State<ZKConfig>) -> Option<NamedFile> {
    if consts.cors || consts.static_files_location == None {
        return None;
    }
    let sf = consts.static_files_location.as_ref();
    NamedFile::open(Path::new(sf.unwrap()).join("index.html"))
        .await
        .ok()
}

#[get("/<file..>", rank = 101)]
pub(crate) async fn static_or_app(file: PathBuf, consts: &State<ZKConfig>) -> Option<NamedFile> {
    if consts.cors || consts.static_files_location == None {
        return None;
    }
    let sf = consts.static_files_location.as_ref();
    match NamedFile::open(Path::new(sf.unwrap()).join(file)).await {
        Ok(n) => Some(n),
        Err(_) => app(&consts).await,
    }
}
