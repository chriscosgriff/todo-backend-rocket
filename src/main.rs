#![feature(plugin)]
#![feature(specialization)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;

use rocket_contrib::JSON;
use rocket::config::{Config, ConfigError, Environment};
use std::env;
use rocket::response::{Responder, Response};
use rocket::http::Status;

#[derive(Serialize)]
struct Todo {
    title: String,
}

struct CORS<R>(R);

impl<'r, R: Responder<'r>> Responder<'r> for CORS<R> {
    default fn respond(self) -> Result<Response<'r>, Status> {
        let mut build = Response::build();
        build.merge(self.0.respond()?);

        build.raw_header("Access-Control-Allow-Origin", "*")
            .raw_header("Access-Control-Allow-Methods",
                        "GET, PUT, PATCH, POST, OPTIONS")
            .raw_header("Access-Control-Allow-Headers", "Content-Type")
            .ok()
    }
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
fn index() -> CORS<JSON<Todo>> {
    CORS(JSON(Todo { title: "Rocket!".to_string() }))
}

fn main() {
    start_server().unwrap();
}
