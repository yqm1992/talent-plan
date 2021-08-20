use std::io::{stdin, Write, Read};
use std::net::{TcpListener, TcpStream};
use std::fs::read;

// fn handle_read_once(stream: &mut TcpStream) -> Option<Vec<u8>> {
//     let s = String::new();
//
//     let mut buf = [0u8;4096];
//     let end_flag = ['\r' as u8, '\n' as u8];
//     let mut readed_len = 0;
//     let mut flag = false;
//
//     while readed_len < buf.len() {
//         readed_len += stream.read(&mut buf[readed_len..]).unwrap();
//         // find "\r\n"
//         if &buf[..readed_len].ends_with(&end_flag) {
//             flag = true;
//             break;
//         }
//     }
//     let remained = Vec::new();
//     if flag {
//         println!("received a message !");
//         // remained.extend_from_slice()
//         Some(remained)
//     } else {
//         println!("error received !");
//         None
//     }
// }


fn write_text(stream: &mut TcpStream, text: &str) {
    let tx = format!("+{}\r\n", text);
    let buf = tx.as_bytes();
    let mut writed_len = 0;

    while writed_len < buf.len() {
        writed_len += stream.write(&buf[writed_len..]).unwrap();
    }
}

fn read_msg(stream: &mut TcpStream) -> Option<String> {
    let mut buf = [0u8;4096];
    let end_flag = ['\r' as u8, '\n' as u8];
    let read_len = stream.read(&mut buf).unwrap();

    if let Some(t) = buf[..read_len].strip_suffix(&end_flag) {
        let s =  String::from_utf8(t.to_vec()).unwrap();
        // println!("received: {}", s);
        Some(s)
    } else {
        println!("Failed to get message from server !");
        None
    }
}

fn main() {
    let mut stream = match TcpStream::connect("127.0.0.1:5000") {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e.to_string());
            std::process::exit(1);
        },
    };

    let mut buf = String::new();
    while let Ok(_) = stdin().read_line(&mut buf) {
        if let Some(text) = buf.strip_suffix('\n') {
            if text == "exit" {
                std::process::exit(0);
            }
            // println!("input: {}", text);
            write_text(&mut stream, text);
            match read_msg(&mut stream) {
                Some(msg) => {
                    if msg.starts_with('+') {
                        println!("{}", msg.strip_prefix('+').unwrap().trim_start())
                    } else if msg.starts_with('-') {
                        println!("{}", msg.strip_prefix('-').unwrap().trim_start())
                    }
                },
                None => break,
            }
        }
        buf.clear();
    }
}