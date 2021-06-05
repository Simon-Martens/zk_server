use chrono::prelude::*;
use git2::{Commit, Error, ObjectType, Oid, Repository, RepositoryInitMode, Signature};
use std::path::Path;

#[derive(Serialize, Debug)]
pub(crate) struct CommitData {
    id: String,
    time: String,
    author: String,
    message: String,
}

pub(crate) struct RepositoryTransaction {
    path: String,
    repo: Repository,
}

impl RepositoryTransaction {
    pub(crate) fn from(path: &str) -> Result<Self, Error> {
        match open_repository(path) {
            Ok(r) => Ok(RepositoryTransaction {
                path: path.to_string(),
                repo: r,
            }),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn find_last_commit(&self) -> Result<CommitData, git2::Error> {
        let obj = self.repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        let commit = obj
            .into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))?;
        Ok(get_commit_data(&commit))
    }

    pub(crate) fn add_and_commit(
        &mut self,
        path: &Path,
        message: &str,
    ) -> Result<Oid, git2::Error> {
        let mut index = self.repo.index()?;
        index.add_path(path)?;
        let oid = index.write_tree()?;
        let signature = Signature::now("Zbigniew Siciarz", "zbigniew@siciarz.net")?; // TODO: User Data
        let obj = self.repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        let parent_commit = obj
            .into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))?;
        let tree = self.repo.find_tree(oid)?;
        self.repo.commit(
            Some("HEAD"), //  point HEAD to our new commit
            &signature,   // author
            &signature,   // committer
            message,      // commit message
            &tree,        // tree
            &[&parent_commit], // parents
        ) 
    }
}

fn open_repository(path: &str) -> Result<Repository, Error> {
    Repository::open(path)
}

fn get_commit_data(commit: &Commit) -> CommitData {
    let timestamp = commit.time().seconds();
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

    CommitData {
        id: format!("{}", commit.id()),
        time: datetime.to_rfc2822(),
        author: format!("{}", commit.author()),
        message: commit.message().unwrap_or("no commit message").to_string(),
    }
}
