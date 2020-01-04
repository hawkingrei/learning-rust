use std::time::Duration;
use std::thread::sleep;

use futures::Future;
use futures::Poll;
use futures::Async;
#[derive(Default)]
struct MyFuture {
    count: u32,
}

impl Future for MyFuture {
    type Item = u32;
    type Error = ();
    
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("Count: {}", self.count);
        
        match self.count {
            3 => Ok(Async::Ready(self.count)),
            _ => {
                self.count += 1;
                Ok(Async::NotReady)
            }
        }
    }
}

struct AddOneFuture<T>(T);

impl<T> Future for AddOneFuture<T>
where
    T: Future,
    T::Item: std::ops::Add<u32, Output=u32>,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(Async::Ready(count)) => {
                println!("Final Count: {}", count + 1);
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(()),
        }
    }
}

fn run<F>(mut f: F) where F: Future<Item=(), Error=()> {
    loop {
        match f.poll() {
            Ok(Async::Ready(_)) => break,
            Ok(Async::NotReady) => (),
            Err(_) => break,
        }
        sleep(Duration::from_millis(100));
    }
}


fn main() {
    let my_future = MyFuture::default();
    run(AddOneFuture(my_future))
}
