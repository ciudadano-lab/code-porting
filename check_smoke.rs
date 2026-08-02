use tinyexpr::te_compile;

fn main() {
    let cases = ["", "1+", "1)", "(1", "1**1", "1*2(+4", "1*2(1+4", "a+5", "!+5", "_a+5", "#a+5", "1^^5", "1**5", "sin(cos5"];
    for expr in cases {
        match te_compile(expr, &[]) {
            Ok(_) => println!("OK {expr:?}"),
            Err(err) => println!("ERR {expr:?} -> {err}"),
        }
    }
}
