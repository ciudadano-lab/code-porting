use std::cell::Cell;
use tinyexpr_rs::{te_compile, te_interp, Variable};

fn main() {
    println!("interp(2 + 3 * 4) = {}", te_interp("2 + 3 * 4").unwrap());

    let x = Cell::new(2.0);
    let y = Cell::new(std::f64::consts::FRAC_PI_2);

    let variables = [
        Variable { name: "x", value: &x },
        Variable { name: "y", value: &y },
    ];

    let expr = te_compile("x^2 + sin(y)", &variables).unwrap();

    println!("compiled expression AST:\n{}", expr.print());
    println!("first eval = {}", expr.eval());

    x.set(3.0);
    println!("after x = 3, eval = {}", expr.eval());
}
