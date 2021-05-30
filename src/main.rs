#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use std::collections::HashSet;
use std::sync::Arc;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::*;
use std::sync::RwLock;
use std::time;

use chrono::Duration;
use chrono::Timelike;
use crypto_hashes::digest::Digest;
use crypto_hashes::digest::consts::*;
use crypto_hashes::digest::generic_array::GenericArray;
use crypto_hashes::digest::generic_array::typenum::UInt;
use crypto_hashes::digest::generic_array::typenum::UTerm;
use hmac::crypto_mac::Output;
use jsonwebtoken::errors::Error;
use jsonwebtoken::errors::ErrorKind;
use rand;
use rand::Rng;
use rocket::State;
use rocket::http::Cookies;
use rocket::http::Cookie;
use rocket::http::Status;
use rocket_contrib::json::{Json, JsonValue};
use rocket::fairing::AdHoc;
use rocket::http::{ContentType};
use rocket::request::*;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::request::FromRequest;
use jsonwebtoken::*;
use crypto_hashes::sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use chrono::{DateTime, TimeZone, Utc};


// TODO: Temporary code
// The type to represent the ID of a message.
type ID = u64;
type Name = String;

// We're going to store all of the messages here. No need for a DB.
type FolderMap = Arc<RwLock<HashMap<u64,StoredFolder>>>;

#[derive(Serialize, Deserialize, Hash)]
struct Folder {
    name: Name
}

#[derive(Serialize, Deserialize, Hash, Clone)]
struct StoredFolder {
    id: ID,
    name: Name
}

// Routes (sorted by prority)
// GET STATIC FILES
#[get("/", rank = 1)]
fn index() -> Option<rocket::response::NamedFile> {
    rocket::response::NamedFile::open(Path::new("/home/simon/repos/zk_vue/dist/index.html")).ok()
}

#[get("/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<rocket::response::NamedFile> {
    rocket::response::NamedFile::open(Path::new("/home/simon/repos/zk_vue/dist/").join(file)).ok()
}

// GET API CALLS
#[get("/<path..>", rank = 3)]
fn all(map: State<FolderMap>, path: PathBuf) -> Json<Vec<StoredFolder>> {
    let hashmap = map.write().unwrap();
    let items_vec: Vec<StoredFolder> = hashmap.values().cloned().collect();
    Json(items_vec)
}

#[get("/", rank = 4)]
fn all_mainpage(map: State<FolderMap>) -> Json<Vec<StoredFolder>> {
    let hashmap = map.write().unwrap();
    let items_vec: Vec<StoredFolder> = hashmap.values().cloned().collect();
    Json(items_vec)
}

// Dynamic POST Routes ("new")
#[post("/post/new/directory", format = "json", data = "<message>", rank = 2)]
fn new(message: Json<Folder>, map: State<FolderMap>) -> JsonValue {
    let mut hashmap = map.write().unwrap();
    let id = calculate_id(&message.0);
    if hashmap.contains_key(&id) {
        json!({
            "status": "error",
            "reason": "ID exists. Error."
        })
    } else {
        hashmap.insert(id, to_stored_message(id, &message.0));
        json!({ "status": "ok" })
    }
}

#[post("/", format="json", data="<message>", rank = 1)] 
fn login_mainpage(message: Json<AuthAttempt>, mut cookies: Cookies, apikey: State<ApiKey>, pw: State<PW>) -> ApiResponse {
    if message.password == pw.0 {
        cookies.add_private(Cookie::new("jwt", issue_token(Claims::new(message.username.clone()), apikey.inner()).unwrap()));
        ApiResponse {
            status: Status::Ok,
            json: json!({"status": "ok"})
        }
    } else {
        ApiResponse {
            status: Status::Forbidden,
            json: json!({"status": "bad username/password"})
        }
    }
}

#[post("/<path..>", format="json", data="<message>", rank = 1)]
fn login(path: PathBuf, message: Json<AuthAttempt>, mut cookies: Cookies, apikey: State<ApiKey>, pw: State<PW>) -> ApiResponse {
    if message.password == pw.0 {
        cookies.add_private(Cookie::new("login data", issue_token(Claims::new(message.username.clone()), apikey.inner()).unwrap()));
        ApiResponse {
            status: Status::Ok,
            json: json!({"status": "ok"})
        }
    } else {
        ApiResponse {
            status: Status::Forbidden,
            json: json!({"status": "bad username/password"})
        }
    }
}

// Dynamic PUT Rules ("edit")

// Dynamic DELETE Rules

// Dynamic OPTION Routes

// Error Catching
#[catch(404)]
fn not_found(req: &rocket::Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

fn main() {
    rocket::ignite()
        .attach(AdHoc::on_attach("Generate Secret", |rocket| {
            let mac: Hmac<Sha256> = Hmac::new_from_slice(&rand::thread_rng().gen::<[u8; 32]>())
                    .expect("Failed to generate Secret. Aborting.");
            Ok(rocket.manage(ApiKey(mac.finalize())))
        }))
        .attach(AdHoc::on_attach("Master PW", |rocket| {
            Ok(rocket.manage(PW("wolfgang".to_string())))
        }))
        .register(catchers![
            not_found
        ])
        .mount("/", routes![
            index, 
            all,
            all_mainpage,
            files,
            new,
            login,
            login_mainpage
        ])
        .manage(FolderMap::default())
        .launch();
}


// Helpers
fn calculate_id<T: Hash>(t: &T) -> u64 {
    let salt: u64 = rand::random();
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.write_u64(salt);
    s.finish()
} 

fn to_stored_message(id: u64, msg: &Folder) -> StoredFolder {
    StoredFolder {
        id,
        name: msg.name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
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
    fn new(user: String) -> Self {
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
            iss: "localhost".to_string(),
            aud: "localhost".to_string(),
            sub: user.clone(),
        }
    }
}

// Token Auth
// Create alias for HMAC-SHA256
type HmacSha256 = Output<Hmac<Sha256>>;
struct PW(String);
struct ApiKey(HmacSha256);
struct JWT(String);

#[derive(Debug)]
enum JWTError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for JWT {
    type Error = JWTError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Option<String> =request.cookies()
            .get_private("jwt")
            .and_then(|cookie| cookie.value().parse().ok());
        if keys == None {
            return Outcome::Failure((Status::Unauthorized, JWTError::Missing));
        }
        let apikey = request.guard::<State<ApiKey>>();
        let validation = validate_token(&keys.unwrap(), &apikey.unwrap());
        match validation {
            Err(e) => Outcome::Failure((Status::Unauthorized, JWTError::Missing)),
            Ok(n) => Outcome::Success(JWT(n.claims.sub))
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct AuthAttempt {
    username: String,
    password: String
}

fn issue_token(claims: Claims, key: &ApiKey) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&key.0.clone().into_bytes())
    )
}

fn validate_token(token: &String, key: &ApiKey) ->  Result<TokenData<Claims>, jsonwebtoken::errors::Error>  {
    let mut validation = Validation {
        leeway: 180,
        validate_nbf: true,
        validate_exp: true,
        iss: Some("localhost".to_string()), // TODO: IP-ADRESSE
        ..Default::default()
    };
    decode(&token, &DecodingKey::from_secret(&key.0.clone().into_bytes()), &validation)
}

mod jwt_numeric_date {
    //! Custom serialization of DateTime<Utc> to conform with the JWT spec (RFC 7519 section 2, "Numeric Date")
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single() // If there are multiple or no valid DateTimes from timestamp, return None
            .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"))
    }
}
#[derive(Debug)]
struct ApiResponse {
    json: JsonValue,
    status: Status,
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
