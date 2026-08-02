use tinyexpr::{te_compile, te_interp};

fn main() {
    let exprs = ["", "1+", "1)", "(1", "1**1", "1*2(+4", "1*2(1+4", "a+5", "!+5", "_a+5", "#a+5", "1^^5", "1**5", "sin(cos5"];
    for e in exprs {
        println!("EXPR: [{e}]");
        match te_interp(e) {
            Ok(v) => println!("  ok: {v}"),
            Err(err) => println!("  err: {err}"),
        }
        match te_compile(e, &[]) {
            Ok(_) => println!("  compile: ok"),
            Err(err) => println!("  compile err: {err}"),
        }
    }
}
