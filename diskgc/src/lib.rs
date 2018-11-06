#![crate_type = "lib"]

extern crate crossbeam;
#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate log;
extern crate tokio_timer;

#[macro_use]
pub mod util;
pub mod worker;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
