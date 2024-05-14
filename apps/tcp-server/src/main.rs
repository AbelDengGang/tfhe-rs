use std::{net::TcpListener, io::{Read, Write}};

fn main() {
    println!("tcp-server: Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    //当有client连接上来时
    for stream in listener.incoming(){
        let mut stream = stream.unwrap();
        println!("Connection established!");
        //创建1k的缓存区
        let mut buffer = [0;1024];
        //读取client发过来的内容
        stream.read(&mut buffer).unwrap();
        //原样送回去(相当于netty的EchoServer)
        stream.write(&mut buffer).unwrap();
    }
}
