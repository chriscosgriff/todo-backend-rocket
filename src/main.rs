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

mod cors;

use cors::CORS;
use rocket_contrib::JSON;
use rocket::config::{Config, ConfigError, Environment};
use rocket::State;
use std::env;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::hash_map::Entry;

struct IdGenerator(AtomicUsize);

impl IdGenerator {
    pub fn next_id(&self) -> usize {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}

#[derive(Deserialize)]
struct NewTodo {
    title: Option<String>,
    completed: Option<bool>,
    order: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn from(id: usize, new_todo: NewTodo) -> Todo {
        Todo {
            title: new_todo.title.or(None),
            completed: new_todo.completed.or(Some(false)),
            // url: Some(format!("http://localhost:{}/todos/{}", get_server_port(), id)),
            url: Some(format!("https://todo-backend-rocket.herokuapp.com/todos/{}", id)),
            order: new_todo.order.or(None),
        }
    }
}

lazy_static! {
  static ref DB: RwLock<HashMap<usize, Todo>> = RwLock::new(HashMap::new());
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
    let server = rocket::custom(get_config()?, true)
        .mount("/todos",
               routes![cors, cors_id, list, delete_all, create, read, update, delete])
        .manage(IdGenerator(AtomicUsize::new(1)))
        .launch();
    Ok(server)
}

#[post("/", format = "application/json", data = "<new_todo>")]
fn create(new_todo: JSON<NewTodo>, id_generator: State<IdGenerator>) -> CORS<JSON<Todo>> {
    let id = id_generator.next_id();
    let todo = Todo::from(id, new_todo.0);
    let mut db = DB.write().unwrap();
    db.insert(id, todo.clone());
    CORS(Some(JSON(todo)))
}

#[patch("/<id>", format = "application/json", data = "<updated_todo>")]
fn update(id: usize, updated_todo: JSON<Todo>) -> CORS<JSON<Todo>> {
    let mut db = DB.write().unwrap();
    if let Entry::Occupied(mut o) = db.entry(id) {
        let mut todo = o.get_mut();
        todo.merge(updated_todo.0);
        CORS(Some(JSON(todo.clone())))
    } else {
        CORS(None)
    }
}

#[get("/<id>")]
fn read(id: usize) -> CORS<JSON<Todo>> {
    let db = DB.read().unwrap();
    let todo = db.get(&id).unwrap();
    CORS(Some(JSON(todo.clone())))
}

#[get("/")]
fn list() -> CORS<JSON<Vec<Todo>>> {
    let db = DB.read().unwrap();
    let todos = db.values().cloned().collect();
    CORS(Some(JSON(todos)))
}

#[delete("/" )]
fn delete_all() -> CORS<()> {
    let mut db = DB.write().unwrap();
    db.clear();
    CORS(None)
}

#[delete("/<id>" )]
fn delete(id: usize) -> CORS<()> {
    let mut db = DB.write().unwrap();
    db.remove(&id);
    CORS(None)
}

#[options("/")]
fn cors() -> CORS<()> {
    CORS(None)
}

#[options("/<id>")]
fn cors_id(id: Option<usize>) -> CORS<()> {
    CORS(None)
}
