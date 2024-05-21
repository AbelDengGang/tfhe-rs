use std::{net::{TcpListener,TcpStream}, io::{self,Read, Write,Cursor,Error},thread};
use tfhe::{ set_server_key, ClientKey, FheUint8, ServerKey};
use tfhe::prelude::*;
use drutil::*;


struct GlobalServerCFG{
    str_map:StringMap,   
}

fn handle_client(mut stream: TcpStream) -> Result<(), Error>{

    // 我不知道如何在多线程中共享数据，出于简单考虑，每个线程维护一个config
    let mut global_server_cfg:GlobalServerCFG = GlobalServerCFG{
        str_map:StringMap{
            key_vec:Vec::new(),
            val_vec:Vec::new(),
        }
    };

    let mut client_key:Option<ClientKey> = None;
    loop {
        let mut receive_pack: CommPackage = CommPackage{
            pack_type:PACK_TYPE_UNKNOW,
            obj_number:0,
            buff:Vec::new(),
        };
        let mut send_pack: CommPackage = CommPackage{
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


                to_pack_ack(&String::from("OK"),&mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            PACK_TYPE_OP => {
                let (op,dtype,oprand1,oprand2) = from_pack_op_u16(&receive_pack);
                match op{
                    OP_ADD => {
                        let result = oprand1 + oprand2;
                        let results = vec![result];

                        to_pack_cipthertests(&DataType::CiptherUint16,&results,&mut send_pack);
                        send(&stream,&send_pack).unwrap();
                    },
                    OP_MUL => {
                        let result = &oprand1 * &oprand2;;
                        let results = vec![result];

                        to_pack_cipthertests(&DataType::CiptherUint16,&results,&mut send_pack);
                        send(&stream,&send_pack).unwrap();

                    },
                    _ => {

                        to_pack_ack(&String::from("OK"),&mut send_pack);
                        send(&stream,&send_pack).unwrap();
                    }
                }
            }
            PACK_TYPE_SERVER_KEY => {
                let mut server_key :ServerKey = from_pack_serverkey(&mut receive_pack);
                set_server_key(server_key);

                to_pack_ack(&String::from("OK"),&mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            PACK_TYPE_ADD_ITEM_ASC_STR => {
                let (k, v) = from_pack_add_map_item_asc_str::<FheAsciiString>(&mut receive_pack);
                global_server_cfg.str_map.key_vec.push(k);
                global_server_cfg.str_map.val_vec.push(v);

                to_pack_ack(&String::from("OK"),&mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            PACK_TYPE_QUERY_ASC_STR => {
                let key : FheAsciiString= from_pack_query_asc_str(&mut receive_pack);
                let result_str = fun_querry_asc_string(&global_server_cfg.str_map.key_vec,
                    &global_server_cfg.str_map.val_vec,&key,&client_key);
                to_pack_reply_asc_str(&result_str, &mut send_pack);
                send(&stream,&send_pack).unwrap();
            }
            PACK_TYPE_CLIENT_KEY =>{
                client_key = Some(from_pack_clientkey(&mut receive_pack));
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