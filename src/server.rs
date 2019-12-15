use std::net::{TcpStream, TcpListener};
use std::io::prelude::*;
use std::fs;
use crate::thread_pool::ThreadPool;


pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    
    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "view/hello.html")
    } else { 
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "view/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
