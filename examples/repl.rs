use tinyexpr::te_interp;
use std::env;
use std::io::{self, Write};
use std::process;

fn eval(str_: &str) -> i32 {
    match te_interp(str_) {
        Ok(r) => {
            println!("{}", r);
            0
        }
        Err(err) => {
            println!("Error at position {}", err);
            -1
        }
    }
}

fn readline(prompt: &str) -> Option<String> {
    eprint!("{}", prompt);
    let _ = io::stderr().flush();

    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) => None, // EOF
        Ok(_) => {
            // Strip trailing newline (same as original fgets + manual strip)
            if buf.ends_with('\n') {
                buf.pop();
                if buf.ends_with('\r') {
                    buf.pop();
                }
            }
            if buf.is_empty() {
                None
            } else {
                Some(buf)
            }
        }
        Err(_) => None,
    }
}

fn add_history(_line: &str) {
    // no-op (same as the non-readline fallback)
}

fn repl() {
    loop {
        let line = match readline("> ") {
            Some(l) => l,
            None => break,
        };

        if line == "q" || line == "quit" {
            break;
        }

        if eval(&line) != -1 {
            add_history(&line);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 3 && args[1] == "-e" {
        if eval(&args[2]) == -1 {
            process::exit(1);
        } else {
            process::exit(0);
        }
    } else if args.len() == 1 {
        repl();
        process::exit(0);
    } else {
        println!("Usage: {}", args[0]);
        println!(" {} -e <expression>", args[0]);
        process::exit(1);
    }
}