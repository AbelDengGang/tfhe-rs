use tfhe::{ConfigBuilder, set_server_key, FheUint8, ClientKey, CompressedServerKey};
use tfhe::prelude::*;

fn main() {

    println!("test App start.");
    let config = ConfigBuilder::default().build();

    let client_key= ClientKey::generate(config);
    let compressed_server_key = CompressedServerKey::new(&client_key);

    let gpu_key = compressed_server_key.decompress_to_gpu();

    let clear_a = 27u8;
    let clear_b = 128u8;

    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);

    //Server-side

    set_server_key(gpu_key);
    let result = a + b;

    //Client-side
    let decrypted_result: u8 = result.decrypt(&client_key);

    let clear_result = clear_a + clear_b;

    assert_eq!(decrypted_result, clear_result);
    println!("test App finish.");
}