use crate::git_interact::CommitData;
use crate::state::ApiKey;
use crate::tokens::issue_token;
use crate::tokens::jwt_numeric_date;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use chrono::Timelike;
use chrono::Utc;
use crypto_hashes::sha2::{Digest, Sha256};
use rocket_contrib::json::JsonValue;
use serde::Serialize;

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

impl Default for Claims {
    fn default() -> Self {
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
            iss: String::default(),
            aud: String::default(),
            sub: String::default(),
        }
    }
}

impl Claims {
    pub(crate) fn set_aud(mut self, aud: &str) -> Self {
        self.aud = aud.to_string();
        self
    }

    pub(crate) fn set_sub(mut self, sub: &str) -> Self {
        self.sub = sub.to_string();
        self
    }

    pub(crate) fn set_iss(mut self, iss: &str) -> Self {
        self.iss = iss.to_string();
        self
    }

    pub(crate) fn set_iat_exp_nbf(mut self, duration: i64) -> Self {
        let iat = Utc::now();
        let nbf = Utc::now();
        let exp = iat + Duration::hours(duration);
        let iat = iat
            .date()
            .and_hms_milli(iat.hour(), iat.minute(), iat.second(), 0);
        let exp = exp
            .date()
            .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);
        let nbf = nbf
            .date()
            .and_hms_milli(nbf.hour(), nbf.minute(), nbf.second(), 0);
        self.iat = iat;
        self.nbf = nbf;
        self.exp = exp;
        self
    }

    pub(crate) fn get_sub(&self) -> String {
        self.sub.clone()
    }

    #[allow(unused)] //  TODO check for right aud on edits
    pub(crate) fn get_aud(&self) -> String {
        self.aud.clone()
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum DataType {
    Empty,
    ErrorMessage,
    MD,
    Directory,
}

#[derive(Debug, Serialize)]
pub(crate) struct AppState {
    authorized: bool,
    time: String,
    commit: Option<CommitData>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            authorized: false,
            commit: None,
            time: Local::now().to_rfc2822(),
        }
    }
}

impl AppState {
    pub(crate) fn set_authorized(mut self, authorized: bool) -> Self {
        self.authorized = authorized;
        self
    }

    #[allow(unused)] // TODO Git Integration
    pub(crate) fn set_commit(mut self, commit: Option<CommitData>) -> Self {
        self.commit = commit;
        self
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ResponseBodyGeneric {
    token: Option<String>, // Token, used aginst XSS, must be valid for writing
    hash: String,          // Sha256-Hash of inner JSON for verification purposes
    url: String,           // Url of the Request for Permalink & History
    history: bool,         // true, if this page will be added to history
    apiurl: String,        // Api-URL of the Request
    inner: JsonValue,      // Inner Json
    datatype: DataType,
    appstate: AppState, // State of the reository
}

impl Default for ResponseBodyGeneric {
    fn default() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(JsonValue::default().0.to_string().as_bytes());
        Self {
            datatype: DataType::Empty,
            hash: format!("{:x}", hasher.finalize()),
            url: String::default(),
            history: false,
            inner: JsonValue::default(),
            token: None,
            apiurl: String::default(),
            appstate: AppState::default(),
        }
    }
}

impl ResponseBodyGeneric {
    pub(crate) fn set_inner(mut self, json: JsonValue, datatype: DataType) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(json.0.to_string().as_bytes());

        self.inner = json;
        self.datatype = datatype;
        self.hash = format!("{:x}", hasher.finalize());
        self
    }

    pub(crate) fn set_history(mut self, enable: bool, url: &str) -> Self {
        self.history = enable;
        self.url = url.to_string();
        self
    }

    pub(crate) fn set_appstate(mut self, appstate: AppState) -> Self {
        self.appstate = appstate;
        self
    }

    pub(crate) fn set_apiurl(self, apiurl: &str, key: &ApiKey, claims: &Claims) -> Self {
        self.set_token(&apiurl, &key, &claims)
    }

    fn set_token(mut self, apiurl: &str, key: &ApiKey, claims: &Claims) -> Self {
        self.apiurl = apiurl.to_string();
        self.token = issue_token(&(claims.clone().set_aud(apiurl).set_iat_exp_nbf(12)), key).ok();
        self
    }

    pub(crate) fn json(&self) -> JsonValue {
        json!(self)
    }
}
