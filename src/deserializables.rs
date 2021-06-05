#[derive(Debug, Deserialize)]
pub(crate) struct AuthAttempt {
    pub(crate) username: String,
    pub(crate) password: String,
}

// #[derive(Debug, Deserialize)]
// pub(crate) struct Attempt {
//     pub(crate) source: String,
// }
