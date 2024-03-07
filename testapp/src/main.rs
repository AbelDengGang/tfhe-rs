use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheInt16, FheUint8, FheUint32, FheUint16};
use tfhe::prelude::*;
use std::time::Instant;
use colored::*;
/// Casting test.
/// 
/// 
fn test_cast(){
    println!("test_cast start!");

    let start_time = Instant::now();
    let config = ConfigBuilder::default().build();
    let end_time = Instant::now();
    println!("ConfigBuilder elapsed: {:?}", end_time - start_time);

    // Client-side
    let (client_key, server_key) = generate_keys(config);


    // Casting requires server_key to set
    // (encryptions/decryptions do not need server_key to be set)
    set_server_key(server_key);

    {
        let clear = 12_837u16;
        let a = FheUint16::encrypt(clear, &client_key);

        // Downcasting
        let start_time = Instant::now();
        let a: FheUint8 = a.cast_into();
        let end_time = Instant::now();
        println!("FheUint16 cast_into FheUint8 elapsed: {:?}", end_time - start_time);
        let da: u8 = a.decrypt(&client_key);
        assert_eq!(da, clear as u8);

        // Upcasting
        let start_time = Instant::now();
        let a: FheUint32 = a.cast_into();
        let end_time = Instant::now();
        println!("FheUint8 cast_into FheUint32 elapsed: {:?}", end_time - start_time);
        let da: u32 = a.decrypt(&client_key);
        assert_eq!(da, (clear as u8) as u32);
    }


    {
        let clear = 12_837u16;
        let a = FheUint16::encrypt(clear, &client_key);

        // Upcasting
        let start_time = Instant::now();
        let a = FheUint32::cast_from(a);
        let end_time = Instant::now();
        println!("FheUint32 cast_from FheUint16 elapsed: {:?}", end_time - start_time);
        let da: u32 = a.decrypt(&client_key);
        assert_eq!(da, clear as u32);

        // Downcasting
        let start_time = Instant::now();
        let a = FheUint8::cast_from(a);
        let end_time = Instant::now();
        println!("FheUint8 cast_from FheUint32 elapsed: {:?}", end_time - start_time);
        let da: u8 = a.decrypt(&client_key);
        assert_eq!(da, (clear as u32) as u8);
    }

    println!("{}","cast_into is faster.".bright_yellow());

    {
        let clear = 12_837i16;
        let a = FheInt16::encrypt(clear, &client_key);

        // Casting from FheInt16 to FheUint16
        let a = FheUint16::cast_from(a);
        let da: u16 = a.decrypt(&client_key);
        assert_eq!(da, clear as u16);
    }

}


/// encrypt/decrypt test
/// 
fn test_encrypt(){
    println!("test_encrypt start!");

    let start_time = Instant::now();
    let config = ConfigBuilder::default().build();
    let end_time = Instant::now();
    println!("ConfigBuilder elapsed: {:?}", end_time - start_time);

    // Client-side
    let (client_key, server_key) = generate_keys(config);

    let clear_a = 27u8;
    let clear_b = 128u8;

    let start_time = Instant::now();
    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);
    let end_time = Instant::now();
    println!("encrypt 2 FheUint8 elapsed: {:?}", end_time - start_time);

    //Server-side
    set_server_key(server_key);
    let start_time = Instant::now();
    let result = a + b;
    let end_time = Instant::now();
    println!("FheUint8 + FheUint8 elapsed: {:?}", end_time - start_time);

    //Client-side
    let start_time = Instant::now();
    let decrypted_result: u8 = result.decrypt(&client_key);
    let end_time = Instant::now();
    println!("decrypt 1 FheUint8 elapsed: {:?}", end_time - start_time);

    let clear_result = clear_a + clear_b;

    assert_eq!(decrypted_result, clear_result);
    println!("test_encrypt done!");    
}

/// shift test
/// 
fn test_shift(){
    println!("test_shift start!");

    let config = ConfigBuilder::default().build();
    let (client_key, sks) = generate_keys(config);

    set_server_key(sks);

    // This is going to be faster
    let a = FheUint32::try_encrypt(2097152u32, &client_key).unwrap();
    let shift = 1u32;
    let start_time = Instant::now();
    let shifted = a << shift;
    let end_time = Instant::now();
    println!("FheUint32 shift 1 bit with clear elapsed: {:?}", end_time - start_time);
    let clear: u32 = shifted.decrypt(&client_key);
    assert_eq!(clear, 2097152 << 1);

    let a = FheUint32::try_encrypt(2097152u32, &client_key).unwrap();
    let shift = 2u32;
    let start_time = Instant::now();
    let shifted = a << shift;
    let end_time = Instant::now();
    println!("FheUint32 shift 2 bit with clear elapsed: {:?}", end_time - start_time);
    let clear: u32 = shifted.decrypt(&client_key);
    assert_eq!(clear, 2097152 << 2);


    // This is going to be slower
    let a = FheUint32::try_encrypt(2097152u32, &client_key).unwrap();
    let shift = FheUint32::try_encrypt_trivial(1u32).unwrap();
    let start_time = Instant::now();
    let shifted = a << shift;
    let end_time = Instant::now();
    println!("FheUint32 shift 1 bit with encrypt_trivial elapsed: {:?}", end_time - start_time);
    let clear: u32 = shifted.decrypt(&client_key);
    assert_eq!(clear, 2097152 << 1);

    let a = FheUint32::try_encrypt(2097152u32, &client_key).unwrap();
    let shift = FheUint32::try_encrypt_trivial(2u32).unwrap();
    let start_time = Instant::now();
    let shifted = a << shift;
    let end_time = Instant::now();
    println!("FheUint32 shift 2 bit with encrypt_trivial elapsed: {:?}", end_time - start_time);
    let clear: u32 = shifted.decrypt(&client_key);
    assert_eq!(clear, 2097152 << 2);


    println!("test_shift done!");    
}


fn test_mul(){
    println!("test_mul done!");    
    let config = ConfigBuilder::default().build();

    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    {
        let a = FheUint16::encrypt(3u16, &client_key);
        let b = FheUint16::encrypt(37849u16, &client_key);
        let c = FheUint16::encrypt(3u16, &client_key);
    
        let start_time = Instant::now();
        let result = &a * &b;
        let end_time = Instant::now();
        println!("FheUint16 * FheUint16 elapsed: {:?}", end_time - start_time);
        let clear_result: u16 = result.decrypt(&client_key);
        assert_eq!(clear_result, 3u16.wrapping_mul(37849u16));
    
        let result_c = &result * &c;
        let clear_result_c: u16 = result_c.decrypt(&client_key);
        assert_eq!(clear_result_c, 3u16.wrapping_mul(3u16.wrapping_mul(37849u16)));
    
    }

    {
        let a = 3u16;
        let b = FheUint16::encrypt(256u16, &client_key);
        let start_time = Instant::now();
        let result = b * a;
        let end_time = Instant::now();
        println!("FheUint16 * scalar elapsed: {:?}", end_time - start_time);
        let clear_result: u16 = result.decrypt(&client_key);
        assert_eq!(clear_result, a.wrapping_mul(256u16));
    }
    println!("test_mul done!");    
}

fn main() {
    if cfg!(debug_assertions) {
        println!("{}","WARNING:Debugging enabled cause poor performance, please run with \"cargo run --release\"!".yellow());
    }

    println!("main start!");
    test_encrypt();
    test_cast();
    test_shift();
    test_mul();
    println!("main finish!");
}