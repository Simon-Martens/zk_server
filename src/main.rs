#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use crate::state::ZKConfig;
use crypto_hashes::sha2::Sha256;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use hmac::Hmac;
use hmac::Mac;
use hmac::NewMac;
use rand;
use rand::Rng;
use rocket::fairing::AdHoc;

mod deserializables;
mod fairings;
mod functions;
mod requestguards;
mod responders;
mod routes_catchers;
mod routes_get;
mod routes_options;
mod routes_post;
mod routes_static_get;
mod serializables;
mod state;
mod tokens;

fn main() {
    rocket::ignite()
        .mount(
            "/api", // TODO: Allow setting this in ZK.toml
            routes![
                routes_get::api,
                routes_get::api_index,
                routes_post::login_mainpage,
                routes_post::login
            ],
        )
        .attach(AdHoc::on_attach("Generate Secret", |rocket| {
            Ok(rocket.manage(state::ApiKey(generate_hmac().finalize())))
        }))
        .attach(AdHoc::on_attach("Parse options", |rocket| {
            Ok(parse_options(rocket))
        }))
        .register(catchers![routes_catchers::not_found])
        .attach(fairings::Gzip)
        .attach(fairings::Caching)
        .attach(fairings::XClacksOverhead)
        .launch();
}

fn read_config() -> ZKConfig {
    let figment = Figment::new()
        .merge(Env::prefixed("ZK_"))
        .merge(Toml::file("./ZK.toml"));
    let config: ZKConfig = figment.extract().unwrap();
    config
}

fn generate_hmac() -> Hmac<Sha256> {
    return Hmac::new_from_slice(&rand::thread_rng().gen::<[u8; 32]>())
        .expect("Failed to generate Secret. Aborting.");
}

fn parse_options(rocket: rocket::Rocket) -> rocket::Rocket {
    let config = read_config();
    let rocket = match config.cors {
        true => rocket
            .attach(fairings::CORS {
                origin: config
                    .cors_origin
                    .as_ref()
                    .expect(
                        "For CORS to be enabled you have to set cors_origin in your preferences.",
                    )
                    .clone(),
            })
            .mount(
                "/",
                routes![routes_options::options, routes_options::options_mainpage],
            ),
        false => rocket.mount(
            "/",
            routes![routes_static_get::app, routes_static_get::static_or_app,],
        ),
    };
    return rocket.manage(config);
}
