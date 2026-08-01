// Port of repl.c (readline support omitted; uses plain stdin like the
// original's non-USE_READLINE fallback path).

use std::io::{self, Write};

fn eval(s: &str) -> bool {
    match tinyexpr::te_interp(s) {
        Ok(r) => {
            println!("{r}");
            true
        }
        Err(err) => {
            println!("Error at position {err}");
            false
        }
    }
}

fn repl() {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF
        }
        let line = line.trim_end_matches(['\n', '\r']);
        if line == "q" || line == "quit" {
            break;
        }
        eval(line);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "-e" {
        if !eval(&args[2]) {
            std::process::exit(1);
        }
    } else if args.len() == 1 {
        repl();
    } else {
        println!("Usage: {}", args[0]);
        println!("       {} -e <expression>", args[0]);
        std::process::exit(1);
    }
}
