use std::{net::TcpListener, io::{self,Read, Write,Cursor}};
use tfhe::{ConfigBuilder, ServerKey, generate_keys, set_server_key, FheUint8};
use tfhe::prelude::*;
fn main() -> io::Result<()> {
    println!("tcp-server: Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    //当有client连接上来时
    for stream in listener.incoming(){
        println!("Connection established!");
        let mut stream = stream.unwrap();
        let mut buffer = Vec::new();
        println!("Recieving!");
        let mut reading: bool = true;
        let mut size_buffer = [0u8;16];
        stream.read_exact(&mut size_buffer)?;
        let total_size = u128::from_ne_bytes(size_buffer.clone());
        println!("expect size is  {}",total_size);
        println!("{},{},{},{}",size_buffer[0],size_buffer[1],size_buffer[2],size_buffer[3]);

        while reading
        {
            let mut tmp_buffer:Vec<u8> = vec![0; 64*1024];
            let readsize = stream.read(&mut tmp_buffer)?;
            println!("readsize {}",readsize);
            tmp_buffer.truncate(readsize);
            buffer.append(&mut tmp_buffer);
            println!("total size {} / {}",buffer.len() as u128,total_size);
            if buffer.len() as u128 >=  total_size{
                reading = false;
            }
        }
        //let mut buffer = Vec::new();
        //创建1k的缓存区
        //let mut buffer = [0;1024];
        //读取client发过来的内容
        //stream.read(&mut buffer).unwrap();
        //原样送回去(相当于netty的EchoServer)
        //stream.write(&mut buffer).unwrap();
        let result = server_function(&buffer).unwrap();
        println!("Sending!");
        stream.write_all(&result)?;
        stream.flush()?;
        println!("Send!");
    }
    Ok(())
}

fn server_function(serialized_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut serialized_data = Cursor::new(serialized_data);
    println!("Deserializing server key!");
    let server_key: ServerKey = bincode::deserialize_from(&mut serialized_data)?;
    println!("Deserializing server val 1!");
    let ct_1: FheUint8 = bincode::deserialize_from(&mut serialized_data)?;
    println!("Deserializing server val 2!");
    let ct_2: FheUint8 = bincode::deserialize_from(&mut serialized_data)?;

    println!("Set_server_key!");
    set_server_key(server_key);

    println!("Adding!");
    let result = ct_1 + ct_2;

    println!("Serialize!");
    let serialized_result = bincode::serialize(&result)?;

    Ok(serialized_result)
}