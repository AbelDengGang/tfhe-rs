#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
use std::{ io::{self, Cursor, Error,Read, Write}, net::{TcpListener, TcpStream}};
use std::thread;
use std::time;
use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use tfhe::{ConfigBuilder, ServerKey, generate_keys, set_server_key, FheUint8,FheUint16};
use tfhe::{ ClientKey,  FheInt16, FheUint,  FheUint16Id, FheUint32};
use tfhe::prelude::*;



pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[allow(dead_code)]
const PACK_TYPE_UNKNOW :u16 = 0;
const PACK_TYPE_SERVER_KEY :u16 = 1;  // 传输server key
const PACK_TYPE_CIPTHERTEXTS :u16 = 2;// 传输一组相同类型的密文，放在Vec里
const PACK_TYPE_PLAINTTEXTS :u16 = 3; // 传输一组相同类型的明文，放在Vec里
const PACK_TYPE_MESSAGE :u16 = 4;     // 传输一个字符串
const PACK_TYPE_ACK :u16 = 5;         // 如果没有返回值，或者出错了，那么server就会回复一个 ACK, OK或者NG
const PACK_TYPE_OP  :u16 = 8;         // 传输操作符,是U16 定义在
const PACK_TYPE_FUN :u16 = 9;         // 传输操作数1，操作数2，操作符
const PACK_TYPE_KEYS :u16 = 10;       // 传输map映射的key集合，以{{index,类型， 密文},...}的形式传输， index 要唯一。相同index则覆盖
const PACK_TYPE_VALUES :u16 = 11;     // 传输map映射的value集合，以{{index,类型， 密文},...}的形式传输， index 要唯一。相同index则覆盖
const PACK_TYPE_QUERY_KEY :u16 = 12;  // 在map映射中查询，
const PACK_TYPE_IN_PROCESS: u16 = 13; // 表示正在处理过程中，客户端接受到这个数据包后面要继续读，直到收到其他的数据包才表明这次通讯结束
const PACK_TYPE_CLIENT_KEY :u16 = 14;  // 传输client key,测试


const OP_ADD  :u16 = 1;
const OP_MUL  :u16 = 2;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum DataType{
    ClearUint16,
    ClearUint8,
    ClearBool,
    CiptherUint16,
    CiptherUint8,
    CiptherBool,
}

//#[derive(Debug)] // 导出调试信息
#[allow(unused_variables)]
pub struct CommPackage {
    obj_number:u16,// buffer里包含了几个对象，需要执行几次反序列
    pack_type:u16,  // 包的类型
    buff: Vec<u8>,  // 缓冲区
}



fn to_pack_cipthertests<T:Serialize>(dtype: & DataType,data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 2;
    pack.pack_type = PACK_TYPE_CIPTHERTEXTS;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &dtype).unwrap();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

fn from_pack_cipthertests_u16(mut pack:&CommPackage) ->(DataType,Vec<FheUint16>)
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let dtype:DataType = bincode::deserialize_from(&mut serialized_data).unwrap();
    let data : Vec<FheUint16> = bincode::deserialize_from(&mut serialized_data).unwrap();
    (dtype,data)
}


fn to_pack_serverkey<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_SERVER_KEY;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

fn from_pack_serverkey<'de,T>(mut pack:&mut CommPackage) -> T
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let data : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    data
}


fn to_pack_clientkey<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_CLIENT_KEY;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

fn from_pack_clientkey<'de,T>(mut pack:&mut CommPackage) -> T
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let data : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    data
}


fn to_pack_ack<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_ACK;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

fn from_pack_ack<'de,T>(mut data:&'de mut T,mut pack:&mut CommPackage)
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    *data = bincode::deserialize_from(&mut serialized_data).unwrap();
}


fn to_pack_op<T:Serialize,OT:Serialize>(dtype:DataType,op:&T,oprand1:&OT, oprand2:&OT,mut pack:&mut CommPackage){
    pack.obj_number = 4;
    pack.pack_type = PACK_TYPE_OP;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &op).unwrap();
    bincode::serialize_into(&mut pack.buff, &dtype).unwrap();
    bincode::serialize_into(&mut pack.buff, &oprand1).unwrap();
    bincode::serialize_into(&mut pack.buff, &oprand2).unwrap();
}

fn from_pack_op_u16(pack:& CommPackage) ->(u16,DataType,FheUint16,FheUint16){
    let mut serialized_data = Cursor::new(pack.buff.clone());
    let op:u16 = bincode::deserialize_from(&mut serialized_data).unwrap();
    let dtype:DataType = bincode::deserialize_from(&mut serialized_data).unwrap();
    let oprand1:FheUint16 = bincode::deserialize_from(&mut serialized_data).unwrap();
    let oprand2:FheUint16 = bincode::deserialize_from(&mut serialized_data).unwrap();
    (op,dtype,oprand1,oprand2)
}



fn from_pack_msg<'de,T>(mut data:&'de mut T,mut pack:&mut CommPackage)
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    *data = bincode::deserialize_from(&mut serialized_data).unwrap();
}


fn to_pack_msg<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_MESSAGE;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}


pub fn send(mut stream:&TcpStream , package: &CommPackage )->Result<(), Box<dyn std::error::Error>>{
    // 计算包的大小
    let mut steam_size :u128 = package.buff.len() as u128;
    steam_size = steam_size + (std::mem::size_of_val(&package.pack_type) as u128) + (std::mem::size_of_val(&package.obj_number) as u128);

    {
        // 发送包的大小
        let mut buffer = Vec::new();
        bincode::serialize_into(&mut buffer, &steam_size)?;
        stream.write(&buffer)?;

    }

    {
        // 发送obj_number
        let mut buffer = Vec::new();
        bincode::serialize_into(&mut buffer, &package.obj_number)?;
        stream.write(&buffer)?;

    }


    {
        // 发送pack_type
        let mut buffer = Vec::new();
        bincode::serialize_into(&mut buffer, &package.pack_type)?;
        stream.write(&buffer)?;

    }

    stream.write_all(&package.buff)?;
    stream.flush()?;

    Ok(())
}

pub fn receive(mut stream:&TcpStream , mut package: &mut CommPackage )->io::Result<()>{
    
    // 接受包长度
    let mut buffer = [0u8;16];
    stream.read_exact(&mut buffer)?;

    let mut total_size = u128::from_ne_bytes(buffer);
    total_size = total_size - (std::mem::size_of_val(&package.pack_type) as u128) - (std::mem::size_of_val(&package.obj_number) as u128);

    let mut buffer = [0u8;2]; // 如何避免hardcode？
    stream.read_exact(&mut buffer)?;
    package.obj_number = u16::from_ne_bytes(buffer);


    let mut buffer = [0u8;2]; // 如何避免hardcode？
    stream.read_exact(&mut buffer)?;
    package.pack_type = u16::from_ne_bytes(buffer);

    package.buff = Vec::new();
    let mut receiving :bool = true;


    while receiving
    {
        let mut tmp_buffer:Vec<u8> = vec![0; 64*1024];
        let readsize = stream.read(&mut tmp_buffer)?;
        //println!("readsize {}",readsize);
        tmp_buffer.truncate(readsize);
        package.buff.append(&mut tmp_buffer);
        //println!("total size {} / {}",package.buff.len() as u128,total_size);
        if package.buff.len() as u128 >=  total_size{
            receiving = false;
        }
    }

    Ok(())
}


fn handle_client(mut stream: TcpStream) -> Result<(), Error>{
    let mut buf = [0; 512];
    let mut client_ley:ClientKey ;
    let mut _server_key:ServerKey;
    // 实际上并不用这个key，只是为了在下面用到client_ley的时候不要报错说没有初始化
    let config = ConfigBuilder::default().build();
    ( client_ley, _server_key) = generate_keys(config);

    for _ in 0..1000 {
        let mut receive_pack: CommPackage = CommPackage{
            pack_type:PACK_TYPE_UNKNOW,
            obj_number:0,
            buff:Vec::new(),
        };
        receive(&stream,&mut receive_pack)?; // 当接受出错的时候，会直接从这里退出函数

        println!("receive type: {}",receive_pack.pack_type);
        match receive_pack.pack_type{
            PACK_TYPE_MESSAGE => {
                let mut msg:String = String::new();
                from_pack_msg(&mut msg,&mut receive_pack);
                println!("receive message: {}",msg);

                let mut send_pack: CommPackage = CommPackage{
                    pack_type:PACK_TYPE_UNKNOW,
                    obj_number:0,
                    buff:Vec::new(),
                };
                to_pack_ack(&String::from("OK"),&mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            PACK_TYPE_OP => {
                let (op,dtype,oprand1,oprand2) = from_pack_op_u16(&receive_pack);
                println!("receive op: {}",op);
                let oprand1_clr: u16 = oprand1.decrypt(&client_ley);
                let oprand2_clr: u16 = oprand2.decrypt(&client_ley);
                println!("receive op: {},oprand1: {}, oprand2:{}",op,oprand1_clr , oprand2_clr);
                match op{
                    OP_ADD => {
                        let result = oprand1 + oprand2;
                        let results = vec![result];
                        let mut send_pack: CommPackage = CommPackage{
                            pack_type:PACK_TYPE_UNKNOW,
                            obj_number:0,
                            buff:Vec::new(),
                        };
                        to_pack_cipthertests(&DataType::CiptherUint16,&results,&mut send_pack);
                        send(&stream,&send_pack).unwrap();
                    },
                    _ => {
                        let mut send_pack: CommPackage = CommPackage{
                            pack_type:PACK_TYPE_UNKNOW,
                            obj_number:0,
                            buff:Vec::new(),
                        };
                        to_pack_ack(&String::from("OK"),&mut send_pack);
                        send(&stream,&send_pack).unwrap();
                    }
                }
            }
            PACK_TYPE_SERVER_KEY => {
                let mut server_key :ServerKey = from_pack_serverkey(&mut receive_pack);
                set_server_key(server_key);
                let mut send_pack: CommPackage = CommPackage{
                    pack_type:PACK_TYPE_UNKNOW,
                    obj_number:0,
                    buff:Vec::new(),
                };
                to_pack_ack(&String::from("OK"),&mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            PACK_TYPE_CLIENT_KEY =>{
                client_ley = from_pack_clientkey(&mut receive_pack);
                let mut send_pack: CommPackage = CommPackage{
                    pack_type:PACK_TYPE_UNKNOW,
                    obj_number:0,
                    buff:Vec::new(),
                };
                to_pack_ack(&String::from("OK"),&mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            default =>{

            }
        }

    }

    Ok(())
}

fn listen_fn()-> Result<(), Error>{
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();

    for stream in listener.incoming() {
        let stream = stream.expect("failed!");
        println!("Connection established!");
        let handle = thread::spawn(move || {
            handle_client(stream)
		.unwrap_or_else(|error| eprintln!("{:?}", error));
        });

        thread_vec.push(handle);
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("it_works");
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn thread_works() {
        // to see println , run cargo test -- --nocapture
        println!("thread_works");
        // 创建一个新线程，并在其中执行指定的闭包
        let handle = thread::spawn(|| {
            // 在新线程中执行的代码
            listen_fn()

        });

        // 在主线程中执行的代码, 等待子线程运行
        for i in 1..=3 {
            println!("Main thread start: {}", i);
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        let config = ConfigBuilder::default().build();
        let ( client_key, server_key) = generate_keys(config);

        let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();
        let mut send_pack:CommPackage = CommPackage{
            obj_number : 1,
            pack_type:PACK_TYPE_MESSAGE,
            buff : Vec::new(),
        };
        let str:String=String::from("This message from client");

        to_pack_msg(&str,&mut send_pack);
        //bincode::serialize_into(&mut send_pack.buff, &str).unwrap();
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


        to_pack_clientkey(&client_key,&mut send_pack);
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



        let msg1 = 1u16;
        let msg2 = 10u16;
        let value_1 = FheUint16::encrypt(msg1, &client_key);
        let value_2 = FheUint16::encrypt(msg2, &client_key);    


        let op = OP_ADD;
        to_pack_op(DataType::CiptherUint16,&op,&value_1,&value_2,&mut send_pack);
        send(&stream,&send_pack).unwrap();
        receive(&stream,&mut receive_pack).unwrap(); // 当接受出错的时候，会直接从这里退出函数
        if receive_pack.pack_type == PACK_TYPE_ACK{
            from_pack_ack(&mut msg, &mut receive_pack);
            println!("From Server: {}",msg);
        }else{
            let (dtype, results) = from_pack_cipthertests_u16(&receive_pack);
            let result_clr: u16 = results[0].decrypt(&client_key);
            println!("From Server: result len{}, result_clr{}",results.len(),result_clr);        
        }

        // 在主线程中执行的代码, 等待子线程运行
        for i in 1..=3 {
            println!("Main thread end: {}", i);
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        //handle.join().unwrap();
        thread::sleep(std::time::Duration::from_millis(1000));
        drop(handle);


    }
}
