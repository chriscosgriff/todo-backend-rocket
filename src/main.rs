#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::config::{Config, ConfigError, Environment};
use std::env;

fn get_server_port() -> u16 {
    let port = env::var("PORT").unwrap_or(String::new());
    port.parse().unwrap_or(8000)
}

fn get_config() -> Result<Config, ConfigError> {
    Config::build(Environment::active()?)
        .port(get_server_port())
        .finalize()
}

fn takeoff() -> Result<(), ConfigError> {
    Ok(rocket::custom(get_config()?, true)
        .mount("/", routes![index])
        .launch())
}

#[get("/")]
fn index() -> &'static str {
    "Rocket"
}

fn main() {
    takeoff().unwrap();
}
