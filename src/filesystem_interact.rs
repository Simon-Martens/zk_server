use rocket::serde::json::Value;
use rocket::serde::json::serde_json::json;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::io;
use std::path::PathBuf;

#[derive(Serialize)]
pub(crate) struct Directory {
    head: Entry,
    mds: Vec<Entry>,
    dirs: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum FType {
    MDFile,
    Directory,
}

#[derive(Serialize)]
pub(crate) struct Entry {
    pub(crate) name: String,
    #[serde(with = "entry_serialization")]
    pub(crate) data: PathBuf,
    pub(crate) ftype: FType,
    pub(crate) url: PathBuf,
}

impl Entry {
    pub(crate) fn json(&self) -> Value {
        json!(self)
    }
}

impl Directory {
    pub(crate) fn json(&self) -> Value {
        json!(self)
    }
}

#[allow(unused_variables)] // TODO: Fiter implement
pub(crate) fn ls(
    entry: Entry,
    basepath: &PathBuf,
    hidden: bool,
    filter: &str,
) -> io::Result<Directory> {
    let mut mds: Vec<Entry> = Vec::new();
    let mut dirs: Vec<Entry> = Vec::new();
    for entry in read_dir(&entry.data)?
        .filter(|e| e.is_ok())
        .filter_map(|e| e.ok())
        .filter(|e| !(is_hidden(&e) ^ hidden))
    {
        let path = entry.path();
        if let Some(e) = open(
            &path.strip_prefix(&basepath).unwrap().to_path_buf(),
            &basepath,
        ) {
            match e.ftype {
                FType::MDFile => mds.push(e),
                FType::Directory => dirs.push(e),
            }
        }
    }
    Ok(Directory {
        head: entry,
        mds,
        dirs,
    })
}

pub(crate) fn open(url: &PathBuf, basepath: &PathBuf) -> Option<Entry> {
    let mut path: PathBuf = basepath.clone();
    path.push(url);
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    match path {
        e if e.is_dir() => Some(Entry {
            data: e,
            name: filename,
            ftype: FType::Directory,
            url: url.clone(),
        }),
        e if e.is_file() && e.extension().unwrap_or_default() == "md" => Some(Entry {
            data: e,
            name: filename,
            ftype: FType::MDFile,
            url: url.clone(),
        }),
        _ => None,
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    if entry
        .path()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .starts_with(".")
    {
        true
    } else {
        false
    }
}

mod entry_serialization {
    use filetime::set_file_atime;
    use filetime::FileTime;
    use serde::ser::SerializeStruct;
    use serde::{self, Serializer};
    use std::fs::metadata;
    use std::fs::read_to_string;
    use std::path::PathBuf;

    use chrono::prelude::{DateTime, Local};
    use std::time::SystemTime;

    pub(crate) fn serialize<S>(entry: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let metadata = metadata(entry).unwrap();
        let mut state = serializer.serialize_struct("Data", 5)?;
        state.serialize_field("size", &metadata.len())?;
        let t = metadata.created().ok().map(|x| systemtime_to_string(x));
        state.serialize_field("created", &t)?;
        let t = metadata.accessed().ok().map(|x| systemtime_to_string(x));
        state.serialize_field("accessed", &t)?;
        let t = metadata.modified().ok().map(|x| systemtime_to_string(x));
        state.serialize_field("modified", &t)?;
        if metadata.is_dir() {
            set_file_atime(&entry, FileTime::now()).ok();
            // TODO: Serialize Number of Items...
        } else {
            let c = read_to_string(entry).ok();
            state.serialize_field("content", &c)?;
            set_file_atime(&entry, FileTime::now()).ok();
        }
        state.end()
    }

    fn systemtime_to_string(t: SystemTime) -> String {
        let t: DateTime<Local> = t.into();
        t.to_rfc2822()
    }
}
