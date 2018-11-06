#[macro_use]
pub mod macros;
pub mod mpsc;
pub mod time;
pub mod timer;

use std::thread;

pub fn get_tag_from_thread_name() -> Option<String> {
    thread::current()
        .name()
        .and_then(|name| name.split("::").skip(1).last())
        .map(From::from)
}
