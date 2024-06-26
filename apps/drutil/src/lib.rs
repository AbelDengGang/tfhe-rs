#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
use std::{ io::{self, Cursor, Error,Read, Write}, net::{TcpListener, TcpStream}};
use std::thread;
use std::time;
use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheBool, FheUint16, FheUint8, ServerKey};
use tfhe::{ ClientKey,  FheInt16, FheUint,  FheUint16Id, FheUint32};
use tfhe::prelude::*;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[allow(dead_code)]
pub const PACK_TYPE_UNKNOW :u16 = 0;
pub const PACK_TYPE_SERVER_KEY :u16 = 1;  // 传输server key
pub const PACK_TYPE_CIPTHERTEXTS :u16 = 2;// 传输一组相同类型的密文，放在Vec里
pub const PACK_TYPE_PLAINTTEXTS :u16 = 3; // 传输一组相同类型的明文，放在Vec里
pub const PACK_TYPE_MESSAGE :u16 = 4;     // 传输一个字符串
pub const PACK_TYPE_ACK :u16 = 5;         // 如果没有返回值，或者出错了，那么server就会回复一个 ACK, OK或者NG
pub const PACK_TYPE_OP  :u16 = 8;         // 传输操作符,是U16 定义在
pub const PACK_TYPE_FUN :u16 = 9;         // 传输操作数1，操作数2，操作符
pub const PACK_TYPE_KEYS :u16 = 10;       // 传输map映射的key集合，以{{index,类型， 密文},...}的形式传输， index 要唯一。相同index则覆盖
pub const PACK_TYPE_VALUES :u16 = 11;     // 传输map映射的value集合，以{{index,类型， 密文},...}的形式传输， index 要唯一。相同index则覆盖
pub const PACK_TYPE_QUERY_KEY :u16 = 12;  // 在map映射中查询，
pub const PACK_TYPE_IN_PROCESS: u16 = 13; // 表示正在处理过程中，客户端接受到这个数据包后面要继续读，直到收到其他的数据包才表明这次通讯结束
pub const PACK_TYPE_CLIENT_KEY :u16 = 14;  // 传输client key,测试
pub const PACK_TYPE_ADD_ITEM_ASC_STR :u16 = 15; // 传输key和val 都是FheAsciiString的包
pub const PACK_TYPE_QUERY_ASC_STR :u16 = 16; // 查询，key是FheAsciiString
pub const PACK_TYPE_REPLY_ASC_STR :u16 = 17; // 回复，value是FheAsciiString


pub const OP_ADD  :u16 = 1;
pub const OP_MUL  :u16 = 2;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum DataType{
    ClearUint16,
    ClearUint8,
    ClearBool,
    CiptherUint16,
    CiptherUint8,
    CiptherBool,
    CiptherAscStr,
}

//#[derive(Debug)] // 导出调试信息
#[allow(unused_variables)]
pub struct CommPackage {
    pub obj_number:u16,// buffer里包含了几个对象，需要执行几次反序列
    pub pack_type:u16,  // 包的类型
    pub buff: Vec<u8>,  // 缓冲区
}



pub fn to_pack_cipthertests<T:Serialize>(dtype: & DataType,data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 2;
    pack.pack_type = PACK_TYPE_CIPTHERTEXTS;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &dtype).unwrap();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

pub fn from_pack_cipthertests_u16(mut pack:&CommPackage) ->(DataType,Vec<FheUint16>)
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let dtype:DataType = bincode::deserialize_from(&mut serialized_data).unwrap();
    let data : Vec<FheUint16> = bincode::deserialize_from(&mut serialized_data).unwrap();
    (dtype,data)
}


pub fn to_pack_add_map_item_asc_str<T:Serialize>(key:&T,val:&T,mut pack:&mut CommPackage){
    pack.obj_number = 2;
    pack.pack_type = PACK_TYPE_ADD_ITEM_ASC_STR;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &key).unwrap();
    bincode::serialize_into(&mut pack.buff, &val).unwrap();
}


pub fn from_pack_add_map_item_asc_str<'de,T>(mut pack:&mut CommPackage) -> (T,T)
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let key : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    let val : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    (key,val)
}


pub fn to_pack_query_asc_str<T:Serialize>(key:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_QUERY_ASC_STR;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &key).unwrap();
}


pub fn from_pack_query_asc_str<'de,T>(mut pack:&mut CommPackage) -> T
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let key : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    key
}

pub fn to_pack_reply_asc_str<T:Serialize>(val:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_REPLY_ASC_STR;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &val).unwrap();
}


pub fn from_pack_reply_asc_str<'de,T>(mut pack:&mut CommPackage) -> T
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let val : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    val
}

pub fn to_pack_serverkey<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_SERVER_KEY;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

pub fn from_pack_serverkey<'de,T>(mut pack:&mut CommPackage) -> T
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let data : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    data
}


pub fn to_pack_clientkey<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_CLIENT_KEY;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

pub fn from_pack_clientkey<'de,T>(mut pack:&mut CommPackage) -> T
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    let data : T = bincode::deserialize_from(&mut serialized_data).unwrap();
    data
}


pub fn to_pack_ack<T:Serialize>(data:&T,mut pack:&mut CommPackage){
    pack.obj_number = 1;
    pack.pack_type = PACK_TYPE_ACK;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &data).unwrap();
}

pub fn from_pack_ack<'de,T>(mut data:&'de mut T,mut pack:&mut CommPackage)
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    *data = bincode::deserialize_from(&mut serialized_data).unwrap();
}


pub fn to_pack_op<T:Serialize,OT:Serialize>(dtype:DataType,op:&T,oprand1:&OT, oprand2:&OT,mut pack:&mut CommPackage){
    pack.obj_number = 4;
    pack.pack_type = PACK_TYPE_OP;
    pack.buff = Vec::new();
    bincode::serialize_into(&mut pack.buff, &op).unwrap();
    bincode::serialize_into(&mut pack.buff, &dtype).unwrap();
    bincode::serialize_into(&mut pack.buff, &oprand1).unwrap();
    bincode::serialize_into(&mut pack.buff, &oprand2).unwrap();
}

pub fn from_pack_op_u16(pack:& CommPackage) ->(u16,DataType,FheUint16,FheUint16){
    let mut serialized_data = Cursor::new(pack.buff.clone());
    let op:u16 = bincode::deserialize_from(&mut serialized_data).unwrap();
    let dtype:DataType = bincode::deserialize_from(&mut serialized_data).unwrap();
    let oprand1:FheUint16 = bincode::deserialize_from(&mut serialized_data).unwrap();
    let oprand2:FheUint16 = bincode::deserialize_from(&mut serialized_data).unwrap();
    (op,dtype,oprand1,oprand2)
}



pub fn from_pack_msg<'de,T>(mut data:&'de mut T,mut pack:&mut CommPackage)
    where
        T: serde::de::DeserializeOwned,
{

    let mut serialized_data = Cursor::new(pack.buff.clone());
    *data = bincode::deserialize_from(&mut serialized_data).unwrap();
}


pub fn to_pack_msg<T:Serialize>(data:&T,mut pack:&mut CommPackage){
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



// TODO 如何拆分成多个文件？
pub const UP_LOW_DISTANCE: u8 = 32;

#[derive(Serialize, Deserialize)]
pub struct FheAsciiString {
    pub bytes: Vec<FheUint8>,
}

pub struct StringMap{
    pub key_vec:Vec<FheAsciiString>,
    pub val_vec:Vec<FheAsciiString>,
}


fn to_upper(c: &FheUint8) -> FheUint8 {
    c - FheUint8::cast_from(c.gt(96) & c.lt(123)) * UP_LOW_DISTANCE
}

fn to_lower(c: &FheUint8) -> FheUint8 {
    c + FheUint8::cast_from(c.gt(64) & c.lt(91)) * UP_LOW_DISTANCE
}

impl FheAsciiString {
    pub fn encrypt(string: &str, client_key: &ClientKey) -> Self {
        // assert!(
        //     string.chars().all(|char| char.is_ascii()),
        //     "The input string must only contain ascii letters"
        // );

        let fhe_bytes: Vec<FheUint8> = string
            .bytes()
            .map(|b| FheUint8::encrypt(b, client_key))
            .collect();

        Self { bytes: fhe_bytes }
    }

    pub fn decrypt(&self, client_key: &ClientKey) -> String {
        let ascii_bytes: Vec<u8> = self
            .bytes
            .iter()
            .map(|fhe_b| fhe_b.decrypt(client_key))
            .collect();
        String::from_utf8(ascii_bytes).unwrap()
    }

    pub fn to_upper(&self) -> Self {
        Self {
            bytes: self.bytes.iter().map(to_upper).collect(),
        }
    }

    pub fn to_lower(&self) -> Self {
        Self {
            bytes: self.bytes.iter().map(to_lower).collect(),
        }
    }

    pub fn eq(&self, other:&FheAsciiString) -> FheBool{
        let size_eq = self.bytes.len() == other.bytes.len();
        if !size_eq {
            return FheBool::encrypt_trivial(false) 
        }
        
        let mut result = FheBool::encrypt_trivial(true) ;

        let mut index = 0;
        while index < self.bytes.len(){
            let a = &self.bytes[index];
            let b = &other.bytes[index];
            let tmp_result = a.eq(b);
            result = &result & tmp_result;
            index += 1;
        }
        return result

    }
}

pub fn try_debug_output(ciphertext:& FheAsciiString,  client_key:& Option<ClientKey>){
    match client_key{
        None => {}
        Some(ck) =>{
            println!("flat text is {}",ciphertext.decrypt(ck));
        }
    }
}

pub fn fun_querry_asc_string(key_set :&Vec<FheAsciiString>, val_set : & Vec<FheAsciiString>, key : & FheAsciiString, client_key:& Option<ClientKey>) -> FheAsciiString{
    try_debug_output(key,client_key);
    assert_eq!(key_set.len(), val_set.len());
    let set_len = key_set.len();
    let mut index = 0;
    let mut max_string_len = 0;
    while index < set_len {
        if max_string_len < val_set[index].bytes.len(){
            max_string_len = val_set[index].bytes.len();
        }
        index += 1;
    }

    let mut result: FheAsciiString = FheAsciiString{
        bytes : Vec::new(),
    };

    // 初始化一个空字符串
    let mut byte_index = 0;
    let fu80 = FheUint8::encrypt_trivial(0 as u8);
    while byte_index < max_string_len{
        result.bytes.push(fu80.clone());
        byte_index += 1;
    }
    let mut index = 0;
    while index < set_len{
        println!("compare key {index}/{set_len}");
        let bool_result = key_set[index].eq(&key);
        let u8_result: FheUint8 = bool_result.clone().cast_into();
        let mut byte_index = 0;
        while byte_index < max_string_len{
            if byte_index < val_set[index].bytes.len() {
                result.bytes[byte_index] = &result.bytes[byte_index] + (&u8_result * &val_set[index].bytes[byte_index]);
            }
            byte_index += 1;
        }
        index +=1;
    }
    try_debug_output(&result,client_key);

    result
}

// cargo test ascill_string_tests --profile release -- --nocapture 来运行这个模块的测试层序
#[cfg(test)]
mod ascill_string_tests {
    use super::*;

    //cargo test ascill_string_tests::string_encrypt_test --profile release -- --nocapture
    #[test]
    fn string_encrypt_test() {
        println!("string_encrypt_test");
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);

        set_server_key(server_key);
        let my_string = FheAsciiString::encrypt("张三", &client_key);
        let verif_string = my_string.decrypt(&client_key);
        println!("Start string: {verif_string}");
        assert_eq!(verif_string, "张三");


        let my_string = FheAsciiString::encrypt("Hello Deep, how is it going?", &client_key);
        let verif_string = my_string.decrypt(&client_key);
        println!("Start string: {verif_string}");

        let my_string_upper = my_string.to_upper();
        let verif_string = my_string_upper.decrypt(&client_key);
        println!("Upper string: {verif_string}");
        assert_eq!(verif_string, "HELLO DEEP, HOW IS IT GOING?");



        // 序列化测试
        let mut buffer = Vec::new();
        let result  = bincode::serialize_into(&mut buffer, &my_string_upper).unwrap();
        let mut serialized_data = Cursor::new(buffer);
        let receive_str:FheAsciiString = bincode::deserialize_from(&mut serialized_data).unwrap();
        let verif_string = receive_str.decrypt(&client_key);
        println!("Receive string: {verif_string}");
        assert_eq!(verif_string, "HELLO DEEP, HOW IS IT GOING?");
    
    }

    //cargo test ascill_string_tests::string_query_test --profile release -- --nocapture
    #[test]
    fn string_query_test() {
        println!("string_pack_test");
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);

        set_server_key(server_key);

        let mut key_vec:Vec<FheAsciiString> = Vec::new() ;
        let mut val_vec:Vec<FheAsciiString> = Vec::new() ;
        println!("Prepare key_vec");
        let key = FheAsciiString::encrypt("wanger", &client_key);
        key_vec.push(key);
        let key = FheAsciiString::encrypt("zhangsan", &client_key);
        key_vec.push(key);
        let key = FheAsciiString::encrypt("lisi", &client_key);
        key_vec.push(key);
        let key = FheAsciiString::encrypt("张三", &client_key);
        key_vec.push(key);

        println!("Prepare val_vec");
        let val = FheAsciiString::encrypt("ge bi lao wang", &client_key);
        val_vec.push(val);
        let val = FheAsciiString::encrypt("haoren", &client_key);
        val_vec.push(val);
        let val = FheAsciiString::encrypt("lurenjia", &client_key);
        val_vec.push(val);
        let val = FheAsciiString::encrypt("我是张三", &client_key);
        val_vec.push(val);

        println!("Prepare key");
        let key = FheAsciiString::encrypt("wanger", &client_key);
        println!("querrying");
        let result_str = fun_querry_asc_string(&key_vec,&val_vec,&key);
        let mut verif_string = result_str.decrypt(&client_key);
        // 由于比较时填充了0，在检查的时候要去掉、0
        verif_string.retain(|c| c != '\0');
        println!("result  string: {verif_string}");
        assert_eq!(verif_string, "ge bi lao wang");

        println!("Prepare key");
        let key = FheAsciiString::encrypt("zhangsan", &client_key);
        println!("querrying");
        let result_str = fun_querry_asc_string(&key_vec,&val_vec,&key);
        let mut verif_string = result_str.decrypt(&client_key);
        // 由于比较时填充了0，在检查的时候要去掉、0
        verif_string.retain(|c| c != '\0');
        println!("result  string: {verif_string}");
        assert_eq!(verif_string, "haoren");

        println!("Prepare key");
        let key = FheAsciiString::encrypt("张三", &client_key);
        println!("querrying");
        let result_str = fun_querry_asc_string(&key_vec,&val_vec,&key);
        let mut verif_string = result_str.decrypt(&client_key);
        // 由于比较时填充了0，在检查的时候要去掉、0
        verif_string.retain(|c| c != '\0');
        println!("result  string: {verif_string}");
        assert_eq!(verif_string, "我是张三");



    }

    //cargo test ascill_string_tests::string_pack_test --profile release -- --nocapture
    #[test]
    fn string_pack_test() {
        println!("string_pack_test");
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);

        set_server_key(server_key);
        let mut send_pack:CommPackage = CommPackage{
            obj_number : 1,
            pack_type:PACK_TYPE_MESSAGE,
            buff : Vec::new(),
        };

        let key = FheAsciiString::encrypt("zhangsan", &client_key);
        let value = FheAsciiString::encrypt("good man", &client_key);
        {
            to_pack_add_map_item_asc_str(&key,&value,&mut send_pack);

            let (rsv_key, rsv_val)= from_pack_add_map_item_asc_str::<FheAsciiString>(&mut send_pack);
    
            let rsv_key_clear = rsv_key.decrypt(&client_key);
            let rsv_val_clear = rsv_val.decrypt(&client_key);
            assert_eq!(rsv_key_clear, "zhangsan");
            assert_eq!(rsv_val_clear, "good man");
        }

        {

            to_pack_query_asc_str(&key,&mut send_pack);
            let rsv_key= from_pack_query_asc_str::<FheAsciiString>(&mut send_pack);
            let rsv_key_clear = rsv_key.decrypt(&client_key);
            assert_eq!(rsv_key_clear, "zhangsan");
        }

        {
            to_pack_reply_asc_str(&value,&mut send_pack);
            let rsv_value= from_pack_reply_asc_str::<FheAsciiString>(&mut send_pack);
            let rsv_value_clear = rsv_value.decrypt(&client_key);
            assert_eq!(rsv_value_clear, "good man");
        }

    }

    //cargo test ascill_string_tests::string_eq_test --profile release -- --nocapture
    #[test]
    fn string_eq_test() {
        println!("string_eq_test");
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);

        set_server_key(server_key);

        let string1 = FheAsciiString::encrypt("hello", &client_key);
        let string2 = FheAsciiString::encrypt("hellooo", &client_key);
        let result = string1.eq(&string2);
        let clr_reuslt = result.decrypt(&client_key);
        assert_eq!(false, clr_reuslt);


        let string1 = FheAsciiString::encrypt("hello", &client_key);
        let string2 = FheAsciiString::encrypt("hello", &client_key);
        let result = string1.eq(&string2);
        let clr_reuslt = result.decrypt(&client_key);
        assert_eq!(true, clr_reuslt);


        let string1 = FheAsciiString::encrypt("nihao", &client_key);
        let string2 = FheAsciiString::encrypt("hello", &client_key);
        let result = string1.eq(&string2);
        let clr_reuslt = result.decrypt(&client_key);
        assert_eq!(false, clr_reuslt);


        let string1 = FheAsciiString::encrypt("张三", &client_key);
        let string2 = FheAsciiString::encrypt("张三", &client_key);
        let result = string1.eq(&string2);
        let clr_reuslt = result.decrypt(&client_key);
        assert_eq!(false, clr_reuslt);

    }


}