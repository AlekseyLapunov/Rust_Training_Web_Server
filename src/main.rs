use std::{
    fmt::format, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, process
};

use config::Config;

pub mod config;

fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        eprintln!("Problem building config: {err}");
        process::exit(1);
    });

    let listener = TcpListener::bind(format!("{host}:{port}", host = config.host, port = config.port))
        .unwrap_or_else(|err| {
        eprintln!("Problem creating TCP listener: {err}");
        process::exit(1);
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "error.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
