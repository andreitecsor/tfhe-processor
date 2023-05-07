#[allow(unused_imports)]

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use tfhe::prelude::*;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8070")?;
    println!("Server is listening");

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("A client initiated connection");
        std::thread::spawn(move || handle_client(stream?));
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    // read an int value from the stream
    let mut buffer = [0u8; 4];
    stream.read_exact(&mut buffer)?;
    let value = i32::from_le_bytes(buffer);

    // print the int value received from the client
    println!("Received value: {}", value);

    // send a response back to the client
    let response = "Hello, client!";
    stream.write_all(response.as_bytes())?;
    Ok(())
}



