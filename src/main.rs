#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::config::{Config, Environment};
use std::env;

fn get_server_port() -> u16 {
    let port = env::var("PORT").unwrap_or(String::new());
    port.parse().unwrap_or(8000)
}

#[get("/")]
fn index() -> &'static str {
    "Rocket"
}

fn main() {
    let config = Config::build(Environment::active().unwrap())
        .port(get_server_port())
        .finalize()
        .unwrap();
    rocket::custom(config, true).mount("/", routes![index]).launch();
}
