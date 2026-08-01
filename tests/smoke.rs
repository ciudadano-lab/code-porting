use std::cell::Cell;
use tinyexpr_rs::{te_compile, te_interp, Variable};

#[test]
fn parse_and_evaluate_basic_expressions() {
    assert_eq!(te_interp("1 + 2 * 3").unwrap(), 7.0);
    assert_eq!(te_interp("(1 + 2) * 3").unwrap(), 9.0);
    assert_eq!(te_interp("2^3^1").unwrap(), 8.0);
    assert!((te_interp("sqrt(16) + log(e)").unwrap() - 5.0).abs() < 1e-12);
}

#[test]
fn compile_with_bound_variables() {
    let x = Cell::new(3.0);
    let y = Cell::new(std::f64::consts::FRAC_PI_2);
    let variables = [
        Variable { name: "x", value: &x },
        Variable { name: "y", value: &y },
    ];
    let expr = te_compile("x^2 + sin(y)", &variables).unwrap();

    assert!((expr.eval() - (9.0 + y.get().sin())).abs() < 1e-12);
    x.set(5.0);
    assert!((expr.eval() - (25.0 + y.get().sin())).abs() < 1e-12);
}

#[test]
fn function_argument_parsing_and_constants() {
    assert!((te_interp("pi + e").unwrap() - (std::f64::consts::PI + std::f64::consts::E)).abs() < 1e-12);
    assert!((te_interp("fac(5) + ncr(5, 2)").unwrap() - (120.0 + 10.0)).abs() < 1e-12);
}
