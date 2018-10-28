#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate rocket;
extern crate tokio;
use futures::future::lazy;
use futures::Future;
use rocket::config::{Config, Environment};
use tokio::runtime::Runtime;

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

    // Spawn a future onto the runtime
    rt.spawn(lazy(|| {
        rocket::ignite().mount("/", routes![hello]).launch();
        Ok(())
    }));
    rt.spawn(lazy(|| {
        println!("wwz");
        let config = Config::build(Environment::Staging)
            .address("0.0.0.0")
            .port(8787)
            .finalize()
            .unwrap();
        rocket::custom(config, false)
            .mount("/", routes![hello2])
            .launch();
        Ok(())
    }));
    rt.shutdown_on_idle().wait().unwrap();
}
