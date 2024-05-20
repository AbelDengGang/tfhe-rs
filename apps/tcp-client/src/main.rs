use std::net::{TcpStream};
use std::io::{self,Read,Write};
use std::str;
use bincode;
use tfhe::{ConfigBuilder, ServerKey, generate_keys, set_server_key, FheUint8,FheUint16};
use tfhe::{ ClientKey,  FheInt16, FheUint,  FheUint16Id, FheUint32};
use tfhe::prelude::*;
use drutil::*;

struct GlobalCFG{
    client_key: ClientKey,
    server_key: ServerKey,
    server_url: String,
    stream: Option<TcpStream>,
    oprand1: u16,
    oprand2: u16,
    op: u16, // OP_ADD,OP_SUB
}

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

enum Menu {
    Root,
    SubMenuNetwork,
    SubMenuOpration,
}

fn display_menu(menu: &Menu) {
    match menu {
        Menu::Root => {
            println!("Main Menu:");
            println!("1. Network");
            println!("2. Opration");
            println!("3. Exit");
        }
        Menu::SubMenuNetwork => {
            println!("Network:");
            println!("1. Connect");
            println!("2. Set URL");
            println!("3. Go back to Main Menu");
        }
        Menu::SubMenuOpration => {
            println!("Sub Menu B:");
            println!("1. oprand1");
            println!("2. oprand2");
            println!("3. oprantion");
            println!("4. Go back to Main Menu");
        }
    }
}


fn read_input() -> String {
    print!("Enter your choice: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}


fn main_menu(mut cfg:&mut  GlobalCFG) -> Menu {
    let mut menu = Menu::Root;
    loop {
        display_menu(&menu);
        match menu {
            Menu::Root => {
                match read_input().as_str() {
                    "1" => menu = Menu::SubMenuNetwork,
                    "2" => menu = Menu::SubMenuOpration,
                    "3" => return Menu::Root,
                    _ => println!("Invalid option"),
                }
            }
            Menu::SubMenuNetwork => {
                match read_input().as_str() {
                    "1" => {
                        cfg.stream = Some(TcpStream::connect(cfg.server_url.clone()).unwrap());
                        let stream = cfg.stream.as_ref().unwrap();

                        println!("Sending server key");
                        let mut send_pack:CommPackage = CommPackage{
                            obj_number : 1,
                            pack_type:PACK_TYPE_MESSAGE,
                            buff : Vec::new(),
                        };
                    
                        to_pack_serverkey(&cfg.server_key,&mut send_pack);
                        send(&stream,&send_pack).unwrap();

                        let mut receive_pack: CommPackage = CommPackage{
                            pack_type:PACK_TYPE_UNKNOW,
                            obj_number:0,
                            buff:Vec::new(),
                        };
                        receive(&stream,&mut receive_pack).unwrap(); // 当接受出错的时候，会直接从这里退出函数
                        let mut msg = String::new();
                        from_pack_ack(&mut msg, &mut receive_pack);
                        println!("From Server: {}",msg);
                    
                    },
                    "2" => println!("Option A2 selected"),
                    "3" => menu = Menu::Root,
                    _ => println!("Invalid option"),
                }
            }
            Menu::SubMenuOpration => {
                match read_input().as_str() {
                    "1" => {
                        println!("Please input oprand1");
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("无法读取输入");
                        input = input.trim().to_string();
                        cfg.oprand1 = input.parse().unwrap();
                        println!("oprand1 is {}",cfg.oprand1);


                    },
                    "2" => {
                        println!("Please input oprand2");
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("无法读取输入");
                        input = input.trim().to_string();
                        cfg.oprand2 = input.parse().unwrap();
                        println!("oprand2 is {}",cfg.oprand2);
                    },
                    "3" => {
                        println!("Please input OP ADD:{},MUL:{}",OP_ADD,OP_MUL);
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("无法读取输入");
                        input = input.trim().to_string();
                        cfg.op = input.parse().unwrap();
                        println!("OP is {}",cfg.op);



                        let msg1 = cfg.oprand1.clone();
                        let msg2 = cfg.oprand2.clone();
                        let value_1 = FheUint16::encrypt(msg1, &cfg.client_key);
                        let value_2 = FheUint16::encrypt(msg2, &cfg.client_key);    
                
                
                        let op = cfg.op;

                        let mut send_pack:CommPackage = CommPackage{
                            obj_number : 1,
                            pack_type:PACK_TYPE_MESSAGE,
                            buff : Vec::new(),
                        };
                        let stream = cfg.stream.as_ref().unwrap();
                        let mut receive_pack: CommPackage = CommPackage{
                            pack_type:PACK_TYPE_UNKNOW,
                            obj_number:0,
                            buff:Vec::new(),
                        };
                        to_pack_op(DataType::CiptherUint16,&op,&value_1,&value_2,&mut send_pack);
                        send(&stream,&send_pack).unwrap();
                        println!("waiting server reply");
                        receive(&stream,&mut receive_pack).unwrap(); // 当接受出错的时候，会直接从这里退出函数
                        println!("received server reply");
                        if receive_pack.pack_type == PACK_TYPE_ACK{
                            let mut msg = String::new();
                            from_pack_ack(&mut msg, &mut receive_pack);
                            println!("From Server: {}",msg);
                        }else{
                            let (dtype, results) = from_pack_cipthertests_u16(&receive_pack);
                            println!("decrypting reply");
                            let result_clr: u16 = results[0].decrypt(&cfg.client_key);
                            println!("From Server: result is : {}",result_clr);        
                        }
                


                    },
                    "4" => menu = Menu::Root,
                    _ => println!("Invalid option"),
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("tcp-client:Hello, world!");
    println!("tcp-client:Creating key!");
    let config = ConfigBuilder::default().build();
    let ( client_key, server_key) = generate_keys(config);
    let msg1 = 1u16;
    let msg2 = 0u16;
    let mut global_cfg :GlobalCFG = GlobalCFG{
        client_key : client_key,
        server_key : server_key,
        server_url : "127.0.0.1:3000".to_string(),
        stream: None,
        oprand1 : 0,
        oprand2: 0,
        op : OP_ADD,
    };
    main_menu(&mut global_cfg);

    Ok(())
}
