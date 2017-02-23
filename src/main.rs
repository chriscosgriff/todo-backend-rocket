#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;

use rocket_contrib::JSON;
use rocket::config::{Config, ConfigError, Environment};
use std::env;

#[derive(Serialize)]
struct Todo {
    title: String,
}

fn get_server_port() -> u16 {
    let port = env::var("PORT").unwrap_or(String::new());
    port.parse().unwrap_or(8000)
}

fn get_config() -> Result<Config, ConfigError> {
    Config::build(Environment::active()?)
        .port(get_server_port())
        .finalize()
}

fn start_server() -> Result<(), ConfigError> {
    Ok(rocket::custom(get_config()?, true)
        .mount("/", routes![index])
        .launch())
}

#[get("/")]
fn index() -> Option<JSON<Todo>> {
    Some(JSON(Todo { title: "Rocket!".to_string() }))
}

fn main() {
    start_server().unwrap();
}
