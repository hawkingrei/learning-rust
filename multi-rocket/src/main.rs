#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate rocket;
extern crate tokio;
use futures::future::lazy;
use futures::Future;
use rocket::config::{Config, Environment};
use tokio::runtime::Runtime;

use std::thread;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[get("/")]
fn hello2() -> &'static str {
    "Hello, world~~!"
}

fn main() {
    let mut rt = Runtime::new().unwrap();

    let builder = thread::Builder::new().name(format!("server-{}", 1));
    let builder1 = thread::Builder::new().name(format!("server-{}", 2));
    // Spawn a future onto the runtime
    let handler = builder
        .spawn(|| rocket::ignite().mount("/", routes![hello]).launch())
        .unwrap();
    let handler1 = builder1
        .spawn(|| {
            let config = Config::build(Environment::Staging)
                .address("0.0.0.0")
                .port(8787)
                .finalize()
                .unwrap();
            rocket::custom(config, false)
                .mount("/", routes![hello2])
                .launch();
        })
        .unwrap();
    handler.join().unwrap();
    handler1.join().unwrap();
}
