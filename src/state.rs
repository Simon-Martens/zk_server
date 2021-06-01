use hmac::crypto_mac::Output;
use hmac::Hmac;
use crypto_hashes::sha2::Sha256; 

type HmacSha256 = Output<Hmac<Sha256>>;

pub(crate) struct ApiKey(
    pub(crate) HmacSha256
);

pub(crate) struct Consts<'a> {
    pub(crate) static_file_location: &'a str,
    pub(crate) repo_file_location: &'a str,
    pub(crate) hostname: &'a str,
    pub(crate) password: &'a str,
}

pub(crate) const CONSTS: Consts<'static> = Consts {
    static_file_location: "/home/simon/repos/zk_vue/dist/",
    repo_file_location: "/home/simon/repos/notes/",
    hostname: "localhost",
    password: "wolfgang",
};

// const BITS_N_STRINGS: BitsNStrings<'static> = BitsNStrings {
//     mybits: BITS,
//     mystring: STRING,
// };