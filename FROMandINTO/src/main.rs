use std::convert::From;

#[derive(Debug)]
struct Number {
    value: i32,
}

impl From<i32> for Number {
    fn from(item: i32) -> Self {
        Number { value: item }
    }
}

fn main() {
    let mut num = Number::from(30);
    println!("My number is {:?}", num);
    let int = 5;
    num  = int.into();
    println!("My number is {:?}", num);
}
