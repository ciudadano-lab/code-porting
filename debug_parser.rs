use tinyexpr::te_compile;

fn main() {
    let expr = "1+";
    match te_compile(expr, &[]) {
        Ok(_) => println!("ok"),
        Err(err) => println!("err={err}"),
    }
}
