use std::{net::{TcpListener,TcpStream}, io::{self,Read, Write,Cursor,Error},thread};
use tfhe::{ ServerKey,  set_server_key, FheUint8};
use tfhe::prelude::*;
use drutil::*;


fn handle_client(mut stream: TcpStream) -> Result<(), Error>{
    let mut buf = [0; 512];


    loop {
        let mut receive_pack: CommPackage = CommPackage{
            pack_type:PACK_TYPE_UNKNOW,
            obj_number:0,
            buff:Vec::new(),
        };
        let result = receive(&stream,&mut receive_pack); // 当接受出错的时候，会直接从这里退出函数
        match result{
            Err(e) =>{ 
                println!("e:{}",e);
                return Ok(())
            },
            Ok(()) =>{}
        }

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
                    OP_MUL => {
                        let result = &oprand1 * &oprand2;;
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

            _ =>{

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

fn main() -> io::Result<()> {
    println!("tcp-server: Hello, world!");
    let handle = thread::spawn(|| {
        // 在新线程中执行的代码
        listen_fn()

    });
    handle.join();
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