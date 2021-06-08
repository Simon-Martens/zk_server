use crate::filesystem_interact::FType;

#[derive(Debug, Deserialize)]
pub(crate) struct AuthAttempt {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateAttempt {
    ftype: FType,
    options: String,
}