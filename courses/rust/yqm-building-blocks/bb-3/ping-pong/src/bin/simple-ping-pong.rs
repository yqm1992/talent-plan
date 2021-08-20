use std::io::stdin;

fn handle(s: &str) {
    if let Some(target) = s.strip_prefix("PING") {
        let echo = target.trim_start();
        if echo.len() == 0 {
            println!("PONG");
        } else {
            println!("{}", echo);
        }
    }
}

fn main() {
    let mut buf = String::new();
    while let Ok(_) = stdin().read_line(&mut buf) {
        if let Some(t) = buf.strip_suffix('\n') {
            if t == "exit" {
                std::process::exit(0);
            }
            handle(t);
        }
        buf.clear();
    }
}