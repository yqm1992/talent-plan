use std::io::{stdin, Write, Read};
use std::net::{TcpListener, TcpStream};
use std::fs::read;

fn handle_write(stream: &mut TcpStream, text: &String) {
    let tx = format!("{}\r\n", text);
    let buf = tx.as_bytes();
    let mut writed_len = 0;

    while writed_len < buf.len() {
        writed_len += stream.write(&buf[writed_len..]).unwrap();
    }
    println!("replied: {}", text);
}

// read from stream, return msg if stream ends with "\r\n"
fn read_msg(stream: &mut TcpStream) -> Option<String> {
    let mut buf = [0u8;4096];
    let end_flag = ['\r' as u8, '\n' as u8];
    let read_len = stream.read(&mut buf).unwrap();

    if let Some(t) = buf[..read_len].strip_suffix(&end_flag) {
        let s =  String::from_utf8(t.to_vec()).unwrap();
        println!("raw received: {}", s);
        Some(s)
    } else {
        println!("Failed to get message from client !");
        None
    }
}

fn process_msg(msg: &String) -> String {
    if let Some(target) = msg.strip_prefix("+PING") {
        let echo = target.trim_start();
        if echo.len() == 0 {
            "+PONG\r\n".to_string()
        } else {
            format!("+{}\r\n", echo).to_string()
        }
    } else {
        "-Error message\r\n".to_string()
    }
}

fn handle(stream: &mut TcpStream) {
    let mut buf = String::new();
    loop {
        match read_msg(stream) {
            Some(msg) => handle_write(stream, &process_msg(&msg)),
            None => break,
        }
        buf.clear()
    }
}

fn main() {
    let mut ac = TcpListener::bind("127.0.0.1:5000").unwrap();
    for s in ac.incoming() {
        let mut stream = s.unwrap();
        handle(&mut stream)
    }
}
