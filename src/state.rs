use crypto_hashes::sha2::Sha256;
use hmac::crypto_mac::Output;
use hmac::Hmac;
use std::sync::atomic::AtomicUsize;

type HmacSha256 = Output<Hmac<Sha256>>;

pub(crate) struct ApiKey(pub(crate) HmacSha256);

// pub(crate) struct FileCount(pub(crate) AtomicUsize);

#[derive(Deserialize)]
pub(crate) struct ZKConfig {
    pub(crate) static_files_location: Option<String>,
    pub(crate) cors: bool,
    pub(crate) cors_origin: Option<String>,
    pub(crate) repo_files_location: String,
    pub(crate) hostname: String,
    pub(crate) admin_password: String,
}
