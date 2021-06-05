use std::path::PathBuf;
use std::path::Path;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::io;

#[derive(Debug, Serialize)]
pub(crate) enum FType {
    MDFile,
    Directory
}

#[derive(Debug, Serialize)]
pub(crate) struct DirectoryEntry{
    ftype: FType,
    #[serde(with = "dir_entry_serialization")]
    entry: PathBuf
}

pub(crate) fn get_directory_contents(path: &PathBuf, onlyhidden: bool) -> io::Result<Vec<DirectoryEntry>> {
    let mut res: Vec<DirectoryEntry> = Vec::new();
    for entry in read_dir(path)?.filter(|e| e.is_ok()).filter_map(|e| e.ok())  {
        // Basic Pattern matching
        let de: DirectoryEntry = match &entry {
            e if entry.path().is_dir() => DirectoryEntry { ftype: FType::Directory, entry: e.path() },
            e if entry.path().extension().unwrap_or_default() == "md" => DirectoryEntry { ftype: FType::MDFile, entry: e.path() },
            _ => continue,
        };

        // Options
        if onlyhidden && !entry.path().file_name().unwrap_or_default().to_string_lossy().starts_with(".") {
            continue
        } // TODO WHAT IF NOT ONLY HIDDEN AND FILE IS HIDDEN

        res.push(de);
    }
    Ok(res)
}


pub(crate) mod dir_entry_serialization {

    use serde::{self, Serializer};
    use std::fs::DirEntry;
    use serde::ser::SerializeStruct;
    use std::path::PathBuf;

    /// Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub(crate) fn serialize<S>(entry: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Entry", 3)?;
        state.end()
    }
}