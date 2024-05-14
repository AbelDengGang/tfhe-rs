use std::net::{TcpStream};
use std::io::{Read,Write};
use std::str;

fn main() {
    println!("tcp-client:Hello, world!");
    let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();

    stream.write("hello,rust.欢迎使用Rust".as_bytes()).unwrap();

    let mut buffer = [0;1024];

    stream.read(&mut buffer).unwrap();

    println!("Response from server:{:?}",str::from_utf8(&buffer).unwrap().trim_matches('\u{0}'));
}
