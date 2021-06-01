#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rand;
use rand::Rng;
use rocket::fairing::AdHoc;
use crypto_hashes::sha2::Sha256;
use hmac::Hmac;
use hmac::Mac;
use hmac::NewMac;

mod functions;
mod requestguards;
mod responders;
mod routes_catchers;
mod routes_get;
mod routes_post;
mod serializables;
mod tokens;
mod state;
mod fairings;
mod deserializables;

fn main() {
    rocket::ignite()
        .attach(AdHoc::on_attach("Generate Secret", |rocket| {
            let mac: Hmac<Sha256> = Hmac::new_from_slice(&rand::thread_rng().gen::<[u8; 32]>())
                    .expect("Failed to generate Secret. Aborting.");
            Ok(rocket.manage(state::ApiKey(mac.finalize())))
        }))
        .attach(AdHoc::on_attach("Master PW", |rocket| {
            Ok(rocket.manage(state::CONSTS))
        }))
        .register(catchers![
            routes_catchers::not_found
        ])
        .attach(fairings::Gzip)
        .attach(fairings::Caching)
        .mount(
            "/", routes![
            routes_get::app,
            routes_get::static_or_app,
            routes_get::api,
            routes_get::api_index,
            routes_post::login_mainpage,
            routes_post::login
        ])
        .launch();
}