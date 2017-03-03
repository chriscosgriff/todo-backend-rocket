#![feature(plugin)]
#![feature(specialization)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use rocket_contrib::JSON;
use rocket::config::{Config, ConfigError, Environment};
use rocket::response::{Responder, Response};
use rocket::http::Status;
use rocket::State;
use std::env;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::hash_map::Entry;

struct CurrentId(AtomicUsize);

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Todo {
    title: Option<String>,
    completed: Option<bool>,
    url: Option<String>,
    order: Option<usize>,
}

impl Todo {
    pub fn merge(&mut self, other: Todo) {
        if let Some(title) = other.title {
            self.title = Some(title);
        }
        if let Some(completed) = other.completed {
            self.completed = Some(completed);
        }
        if let Some(order) = other.order {
            self.order = Some(order);
        }
    }
}

struct CORS<R>(Option<R>);

impl<'r, R: Responder<'r>> Responder<'r> for CORS<R> {
    default fn respond(self) -> Result<Response<'r>, Status> {
        let mut build = Response::build();
        if let Some(inner_res) = self.0 {
            build.merge(inner_res.respond()?);
        }

        build.raw_header("access-control-allow-origin", "*")
            .raw_header("access-control-Allow-Methods",
                        "OPTIONS, GET, POST, PATCH, DELETE")
            .raw_header("access-control-allow-headers", "Content-Type")
            .ok()
    }
}

lazy_static! {
  static ref DB: Mutex<HashMap<usize, Todo>> = Mutex::new(HashMap::new());
}

fn main() {
    start_server().unwrap();
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
        .mount("/",
               routes![cors, cors_id, list, delete_all, create, read, update, delete])
        .manage(CurrentId(AtomicUsize::new(1)))
        .launch())
}

fn to_new_todo(id: usize, new_todo: Todo) -> Todo {
    Todo {
        title: new_todo.title.or(None),
        completed: new_todo.completed.or(Some(false)),
        url: Some(format!("http://localhost:8000/{}", id)),
        order: new_todo.order.or(None),
    }
}

#[post("/", format = "application/json", data = "<todo>")]
fn create(todo: JSON<Todo>, current_id: State<CurrentId>) -> CORS<JSON<Todo>> {
    let id = current_id.0.fetch_add(1, Ordering::Relaxed);
    let mut db = DB.lock().unwrap();
    let td = db.entry(id).or_insert(to_new_todo(id, todo.into_inner()));
    println!("{:?}", td);
    CORS(Some(JSON(td.clone())))
}

#[patch("/<id>", format = "application/json", data = "<todo_patched>")]
fn update(id: usize, todo_patched: JSON<Todo>) -> CORS<JSON<Todo>> {
    let mut db = DB.lock().unwrap();
    if let Entry::Occupied(mut o) = db.entry(id) {
        let mut todo = o.get_mut();
        todo.merge(todo_patched.into_inner());
        CORS(Some(JSON(todo.clone())))
    } else {
        CORS(None)
    }
}

#[get("/<id>")]
fn read(id: usize) -> CORS<JSON<Todo>> {
    let db = DB.lock().unwrap();
    // handle not found...
    let todo = db.get(&id).unwrap();
    CORS(Some(JSON(todo.clone())))
}

#[get("/")]
fn list() -> CORS<JSON<Vec<Todo>>> {
    let db = DB.lock()
        .unwrap();
    let todos = db.values()
        .cloned()
        .collect();
    CORS(Some(JSON(todos)))
}

#[delete("/" )]
fn delete_all() -> CORS<()> {
    let mut db = DB.lock().unwrap();
    db.clear();
    CORS(None)
}

#[delete("/<id>" )]
fn delete(id: usize) -> CORS<()> {
    let mut db = DB.lock().unwrap();
    db.remove(&id);
    CORS(None)
}

#[options("/")]
fn cors() -> CORS<()> {
    CORS(None)
}

#[options("/<id>")]
fn cors_id(id: usize) -> CORS<()> {
    CORS(None)
}
