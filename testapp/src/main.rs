#![allow(warnings, unused)]

use tfhe::{generate_keys, set_server_key, ClientKey, ConfigBuilder, FheInt16, FheUint, FheUint16, FheUint16Id, FheUint32, FheUint8};
use tfhe::prelude::*;
use std::time::Instant;
use colored::*;
use image::{GenericImageView, RgbaImage, Rgba};

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


fn test_blend_a_pixel(){
    println!("test_blend_a_pixel start!");

    let config = ConfigBuilder::default().build();
    let (client_key, sks) = generate_keys(config);

    set_server_key(sks);
    let alpha: u16= 40;
    let a = FheUint16::try_encrypt(100u16, &client_key).unwrap();
    let b = FheUint16::try_encrypt(50u16, &client_key).unwrap();

    let c = blend_a_pixel(&a,&b,alpha*256/100);

    let clear_c:u16 = c.decrypt(&client_key);
    println!("blend value is {clear_c}");
    println!("test_blend_a_pixel done!");    
}


fn blend_a_pixel(a:&FheUint<FheUint16Id>, b:&FheUint<FheUint16Id>,alpha:u16)->FheUint<FheUint16Id>{
    let left:u16 = 256 - alpha;
    let top = a * alpha;
    let bottom = b * left;
    let mut result = top + bottom;
    result = result >> 8u16;
    return result;
}

fn test_load_picture(){
    let mut img = image::open("G0024517.JPG").unwrap();
    println!("Image dimensions: {:?}", img.dimensions());
   // let img_0 = img.into_rgb8();
    let cropped_0 = img.crop(10,10,640,480);
    let cropped_1 = img.crop(700, 500, 640, 480);
    let pix = cropped_0.get_pixel(0, 0);
    println!("{:?}",pix);
    let r = pix[0] as u16;
    let g = pix[1] as u16;
    let b = pix[2] as u16;
    println!("r:{r},g:{g},b:{b}");
    let _ = cropped_0.save("output_0.jpg");
    let _ = cropped_1.save("output_1.jpg");
}

fn test_blend_picture(){
    println!("test_blend_picture start!");
    let mut img_A = image::open("A.JPG").unwrap();
    let mut img_B = image::open("B.JPG").unwrap();
    assert_eq!(img_A.dimensions(),img_B.dimensions());
    let (w,h) = img_A.dimensions();
    let mut img_output = RgbaImage::new(w, h);

    let config = ConfigBuilder::default().build();
    let (client_key, sks) = generate_keys(config);

    set_server_key(sks);
    let alpha: u16= 40;
    let start_time = Instant::now();

    for x in 0..w{
        for y in 0..h{
            let pix_a = img_A.get_pixel(x, y);
            let pix_b = img_B.get_pixel(x, y);
            let mut pix_blend = [0u8,0u8,0u8,255u8];
            for i in 0..3{
                println!("blending {x},{y}:[{i}]");
                let clear_a = pix_a[i] as u16;
                let clear_b = pix_b[i] as u16;

                let a = FheUint16::try_encrypt(clear_a, &client_key).unwrap();
                let b = FheUint16::try_encrypt(clear_b, &client_key).unwrap();
            
                let c = blend_a_pixel(&a,&b,alpha*256/100);
            
                let clear_c:u16 = c.decrypt(&client_key);
                let clear_c_u8 = clear_c as u8;
                pix_blend[i] = clear_c_u8;

            }
            img_output.put_pixel(x, y, Rgba(pix_blend));
        }
    }
    let end_time = Instant::now();
    println!("blend 2 {w} * {h} elapsed: {:?}", end_time - start_time);

    let _ = img_output.save("blend.png");
    println!("test_blend_picture done!");    

}

fn main() {
    if cfg!(debug_assertions) {
        println!("{}","WARNING:Debugging enabled cause poor performance, please run with \"cargo run --release\"!".yellow());
    }

    println!("main start!");
    // test_encrypt();
    // test_cast();
    // test_shift();
    // test_mul();
    // test_blend_a_pixel();
    //test_load_picture();
    test_blend_picture();
    println!("main finish!");
}