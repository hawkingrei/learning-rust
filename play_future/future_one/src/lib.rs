extern crate futures;

use std::fmt::Error;

use futures::future::ok;
use futures::Future;

fn my_fn() -> Result<u32, Box<Error>> {
    Ok(100)
}

fn my_fn_squared(i: u32) -> Result<u32, Box<Error>> {
    Ok(i * i)
}

fn my_fut_squared(i: u32) -> impl Future<Item = u32, Error = Box<Error>> {
    ok(i * i)
}

fn my_fut() -> impl Future<Item = u32, Error = Box<Error>> {
    ok(100)
}

#[cfg(test)]
fn it_works() {
    let retval = my_fn().unwrap();
    println!("{:?}", retval);
    let retval2 = my_fn_squared(retval).unwrap();
    println!("{:?}", retval2);
}
