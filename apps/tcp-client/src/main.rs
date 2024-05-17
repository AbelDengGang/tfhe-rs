use std::net::{TcpStream};
use std::io::{Read,Write};
use std::str;
use bincode;
use tfhe::{ConfigBuilder, ServerKey, generate_keys, set_server_key, FheUint8,FheUint16};
use tfhe::{ ClientKey,  FheInt16, FheUint,  FheUint16Id, FheUint32};
use tfhe::prelude::*;
use drutil::*;


fn test_eq(){

    let (client_key, server_key) = generate_keys(ConfigBuilder::default());

    set_server_key(server_key);
    let a = FheUint16::encrypt(5u16, &client_key);
    let b = FheUint16::encrypt(2u16, &client_key);

    let result = a.eq(&b);
    let c: FheUint<FheUint16Id> = result.clone().cast_into();
    let clear_c :u16= c.decrypt(&client_key);
    println!("decrypted c {}",clear_c);
    let decrypted = result.decrypt(&client_key) as u8;
    println!("decrypted  {}",decrypted);


    let a = FheUint16::encrypt(2u16, &client_key);
    let b = FheUint16::encrypt(2u16, &client_key);

    let result = a.eq(&b);
    let decrypted = result.decrypt(&client_key) as u8;
    println!("decrypted  {}",decrypted);



}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("tcp-client:Hello, world!");
    println!("tcp-client:Creating key!");
    let config = ConfigBuilder::default().build();
    let ( client_key, server_key) = generate_keys(config);
    let msg1 = 1u16;
    let msg2 = 0u16;

    let mut send_pack:CommPackage = CommPackage{
        obj_number : 1,
        pack_type:drutil::PACK_TYPE_MESSAGE,
        buff : Vec::new(),
    };
    println!("tcp-client:encrypting!");
    let value_1 = FheUint16::encrypt(msg1, &client_key);
    println!("tcp-client:encrypting!");
    let value_2 = FheUint16::encrypt(msg2, &client_key);

    let mut serialized_data = Vec::new();
    println!("tcp-client:serializing server_key! {}",serialized_data.len());
    bincode::serialize_into(&mut serialized_data, &server_key)?;
    println!("tcp-client:serializing value_1! {}",serialized_data.len());
    bincode::serialize_into(&mut serialized_data, &value_1)?;
    println!("tcp-client:serializing value_2! {}",serialized_data.len());
    bincode::serialize_into(&mut serialized_data, &value_2)?;


    let steam_size :u128 = serialized_data.len() as u128;
    let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();
    let mut send_pack:CommPackage = CommPackage{
        obj_number : 1,
        pack_type:PACK_TYPE_MESSAGE,
        buff : Vec::new(),
    };

    to_pack_serverkey(&server_key,&mut send_pack);
    send(&stream,&send_pack).unwrap();
    // 等待新线程执行完成
    let mut receive_pack: CommPackage = CommPackage{
        pack_type:PACK_TYPE_UNKNOW,
        obj_number:0,
        buff:Vec::new(),
    };
    receive(&stream,&mut receive_pack).unwrap(); // 当接受出错的时候，会直接从这里退出函数
    let mut msg = String::new();
    from_pack_ack(&mut msg, &mut receive_pack);
    println!("From Server: {}",msg);



    //assert_eq!(output, msg1 + msg2);
    println!("OK!");
    Ok(())
}
