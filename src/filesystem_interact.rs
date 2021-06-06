use std::fs::metadata;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::Metadata;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize)]
pub(crate) struct DirectoryEntries {
    head: DirectoryEntry,
    mds: Vec<DirectoryEntry>,
    dirs: Vec<DirectoryEntry>,
}

#[derive(Serialize)]
pub(crate) enum FType {
    MDFile,
    Directory
}

#[derive(Serialize)]
pub(crate) struct DirectoryEntry {
    name: String,
    #[serde(with = "dir_entry_serialization")]
    path: PathBuf,
    ftype: FType,
}

pub(crate) fn get_all_directory(p: DirectoryEntry, hidden: bool) -> io::Result<DirectoryEntries> {
    let mut mds: Vec<DirectoryEntry> = Vec::new();
    let mut dirs: Vec<DirectoryEntry> = Vec::new();
    for entry in read_dir(&p.path)?
        .filter(|e| e.is_ok())
        .filter_map(|e| e.ok())
        .filter(|e| !(is_hidden(&e) ^ hidden))
    {
        let path = entry.path();
        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        match path {
            e if e.is_dir() => dirs.push(DirectoryEntry {
                path: e,
                name: filename,
                ftype: FType::Directory
            }),
            e if e.extension().unwrap_or_default() == "md" => mds.push(DirectoryEntry {
                path: e,
                name: filename,
                ftype: FType::MDFile
            }),
            _ => continue,
        };
    }
    Ok(DirectoryEntries {
        head: p,
        mds,
        dirs
    })
}

pub(crate) fn get_file(path: PathBuf) -> io::Result<DirectoryEntry> {
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    Ok(DirectoryEntry {
        ftype: if path.is_dir() { FType::Directory } else { FType::MDFile },
        path: path,
        name: filename,
    })
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

mod dir_entry_serialization {
    use chrono::Date;
    use filetime::set_file_atime;
    use filetime::FileTime;
    use serde::ser::SerializeStruct;
    use serde::{self, Serializer};
    use std::fs::metadata;
    use std::fs::read_to_string;
    use std::fs::DirEntry;
    use std::path::PathBuf;

    use chrono::prelude::{DateTime, Utc};
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
            set_file_atime(&entry, FileTime::now());
            // TODO: Serialize Number of Items...
        } else {
            let c = read_to_string(entry).ok();
            state.serialize_field("content", &c);
            set_file_atime(&entry, FileTime::now());
        }
        state.end()
    }

    fn systemtime_to_string(t: SystemTime) -> String {
        let t: DateTime<chrono::Local> = t.into();
        t.to_rfc2822()
    }
}
