#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use std::sync::Arc;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::*;
use std::sync::RwLock;

use rand;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

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
// Static routes
#[get("/", rank = 1)]
fn index() -> Option<rocket::response::NamedFile> {
    rocket::response::NamedFile::open(Path::new("/home/simon/repos/zk_web/breadboard/index.html")).ok()
}

#[get("/static/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<rocket::response::NamedFile> {
    rocket::response::NamedFile::open(Path::new("/home/simon/repos/zk_web/breadboard/").join(file)).ok()
}

// Dynamic GET Routes
#[get("/hello/<name>")]
fn hello(name: &rocket::http::RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}

#[get("/get/<path..>")]
fn all(map: State<FolderMap>, path: PathBuf) -> Json<Vec<StoredFolder>> {
    let hashmap = map.write().unwrap();
    let items_vec: Vec<StoredFolder> = hashmap.values().cloned().collect();
    Json(items_vec)
}

// Dynamic POST Routes ("new")
#[post("/post/new/directory", format = "json", data = "<message>")]
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

// Dynamic PUT Rules ("edit")

// Dynamic DELETE Rules

// Error Catching
#[catch(404)]
fn not_found(req: &rocket::Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

fn main() {
    rocket::ignite()
        .register(catchers![
            not_found
        ])
        .mount("/", routes![
            index, 
            files,
            hello,
            all,
            new
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