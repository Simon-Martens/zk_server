use crate::state::ApiKey;
use crate::state::ZKConfig;
use crate::tokens::issue_token;
use crate::tokens::jwt_numeric_date;
use chrono::DateTime;
use chrono::Duration;
use chrono::Timelike;
use chrono::Utc;
use crypto_hashes::sha2::{Sha256, Sha512, Digest};
use rand::random;
use rocket::State;
use rocket_contrib::json::JsonValue;
use serde::Serialize;
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    iss: String, // Optional. Issuer.
    #[serde(with = "jwt_numeric_date")]
    iat: DateTime<Utc>, // Optional. Issued at (as UTC timestamp)
    #[serde(with = "jwt_numeric_date")]
    exp: DateTime<Utc>, // Required. Expiration time.
    #[serde(with = "jwt_numeric_date")]
    nbf: DateTime<Utc>, // Optional. When the Key starts working.
    sub: String, // Optional. Subject (whom token refers to)
    aud: String, // Optional. Identfies the Subject further (constructed and verified in header)
}

impl Claims {
    pub(crate) fn new(user: &str, consts: &State<ZKConfig>) -> Self {
        let iat = Utc::now();
        let nbf = Utc::now();
        let exp = iat + Duration::days(1);
        let iat = iat
            .date()
            .and_hms_milli(iat.hour(), iat.minute(), iat.second(), 0);
        let exp = exp
            .date()
            .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);
        let nbf = nbf
            .date()
            .and_hms_milli(nbf.hour(), nbf.minute(), nbf.second(), 0);
        Self {
            iat,
            nbf,
            exp,
            iss: consts.hostname.clone(),
            aud: consts.hostname.clone(),
            sub: user.to_string(),
        }
    }

    pub(crate) fn set_aud(mut self, aud: &str) -> Self {
        self.aud = aud.to_string();
        self
    }

    pub(crate) fn get_sub(&self) -> String {
        self.sub.clone()
    }
}
#[derive(Debug, Serialize)]
pub(crate) enum DataType {
    Empty,
    Ignore
}

#[derive(Debug, Serialize)]
pub(crate) struct ResponseBodyGeneric {
    token: String,    // Token, used aginst XSS, must be valid for writing
    hash: String,     // Sha256-Hash of inner JSON for verification purposes
    url: String,      // Url of the Request for Permalink & History
    history: bool,    // true, if this page will be added to history
    apiurl: String,   // Api-URL of the Request
    inner: JsonValue, // Inner Json
    datatype: DataType,
    appstate: ResponseAppState, // State of the reository
}

impl ResponseBodyGeneric {
    pub(crate) fn empty(apiurl: &str, key: &ApiKey, claims: &Claims) -> Self {
        Self {
            datatype: DataType::Empty,
            hash: "unset".to_string(),
            url: "unset".to_string(),
            history: false,
            inner: json! {""},
            token: issue_token(claims.clone().set_aud(apiurl), key).unwrap(),
            apiurl: apiurl.to_string(),
            appstate: ResponseAppState {
                // TODO: Initialize here
                repo_hash: "unimplemented".to_string(),
            },
        }
    }

    pub(crate) fn inner(mut self, json: JsonValue, datatype: DataType) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(json.0.to_string().as_bytes());

        self.inner = json;
        self.datatype = datatype;
        self.hash = format!("{:x}", hasher.finalize());
        self
    }

    pub(crate) fn enable_history(mut self, enable: bool, url: &str) -> Self {
        self.history = enable;
        self.url = url.to_string();
        self
    }

    // pub(crate) fn send(inner: T) -> Self {
    //     Self {
    //         token: "unimplemented".to_string(),
    //         datatype: DataType::Empty,
    //         hash: "unimplmented".to_string(),
    //         url: "unimplemented".to_string(),
    //         history: false,
    //         apiurl: "unimplemented".to_string(),
    //         inner,
    //         appstate: ResponseAppState {
    //             repo_hash: "".to_string(),
    //         },
    //     }
    // }

    pub(crate) fn json(&self) -> JsonValue {
        json!(self)
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ResponseAppState {
    pub(crate) repo_hash: String, // TODO: Hash of latest Commit
                                  // TODO: DateTime of latest commit
}
fn rand_string() -> String {
    (0..8)
        .map(|_| (0x20u8 + (random::<f32>() * 96.0) as u8) as char)
        .collect()
}
