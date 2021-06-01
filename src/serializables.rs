use std::hash::Hash;
use chrono::Duration;
use chrono::Timelike;
use chrono::DateTime;
use chrono::Utc;
use rocket::State;
use serde::Serialize;
use crate::tokens::jwt_numeric_date;
use crate::state::Consts;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    iss: String,         // Optional. Issuer.
    #[serde(with = "jwt_numeric_date")]
    iat: DateTime<Utc>,   // Optional. Issued at (as UTC timestamp)
    #[serde(with = "jwt_numeric_date")]
    exp: DateTime<Utc>,   // Required. Expiration time.
    #[serde(with = "jwt_numeric_date")]
    nbf: DateTime<Utc>,  // Optional. When the Key starts working.
    sub: String,         // Optional. Subject (whom token refers to)
    aud: String,         // Optional. Identfies the Subject further (constructed and verified in header)
}

impl Claims {
    pub(crate) fn new(user: String, consts: State<Consts>) -> Self {
        let iat = Utc::now();
        let nbf = Utc::now();
        let exp = iat + Duration::days(1);        
        let iat = iat.date().and_hms_milli(iat.hour(), iat.minute(), iat.second(), 0);
        let exp = exp.date().and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);
        let nbf = nbf.date().and_hms_milli(nbf.hour(), nbf.minute(), nbf.second(), 0);
        Self {
            iat,
            nbf,
            exp,
            iss: consts.hostname.to_string(),
            aud: consts.hostname.to_string(),
            sub: user.clone(),
        }
    }

    pub(crate) fn get_sub(self) -> String {
        self.sub.clone()
    }
}


// #[derive(Debug, Serialize, Deserialize)]
// pub(crate) enum ResponseStatus {
//     Ok,
//     AuthError,
//     Working,
//     BadRequest
// }

#[derive(Debug, Serialize)]
pub(crate) struct ResponseBodyGeneric<T: Serialize> {
    pub(crate) token: String,               // Token, used aginst XSS, must be valid for writing
    pub(crate) hash: String,                // Sha256-Hash of inner JSON for verification purposes
    pub(crate) url: String,                 // Url of the Request for Permalink & History
    pub(crate) history: bool,               // true, if this page will be added to history
    pub(crate) apiurl: String,              // Api-URL of the Request
    pub(crate) inner: T,                    // Inner Json
    pub(crate) appstate: ResponseAppState,  // State of the reository
}

impl<T> ResponseBodyGeneric<T> where T: Serialize {
    pub(crate) fn lazy(inner: T) -> Self {
        Self {
            token: "unimplemented".to_string(),
            hash: "unimplmented".to_string(),
            url: "unimplemented".to_string(),
            history: false,
            apiurl: "unimplemented".to_string(),
            inner,
            appstate: ResponseAppState {
                repo_hash: "".to_string(),
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ResponseAppState {
    pub(crate) repo_hash: String,           // TODO: Hash of latest Commit
                                            // TODO: DateTime of latest commit
}