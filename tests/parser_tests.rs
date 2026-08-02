use std::os::raw::c_void;
use std::ptr;

use tinyexpr::{te_compile, te_eval_expr, TeVariable, TE_VARIABLE};

fn assert_eval(expr: &str, expected: f64, variables: &[TeVariable]) {
    let compiled = te_compile(expr, variables).unwrap_or_else(|err| {
        panic!("failed to compile {expr:?}: {err}")
    });
    let actual = te_eval_expr(&compiled);
    assert!(
        (actual - expected).abs() < 1e-10,
        "expected {expr:?} to evaluate to {expected}, got {actual}"
    );
}

fn assert_error(expr: &str, variables: &[TeVariable]) {
    assert!(
        te_compile(expr, variables).is_err(),
        "expected {expr:?} to fail compilation"
    );
}

fn variable_lookup(x: f64, y: f64) -> [TeVariable; 2] {
    [
        TeVariable {
            name: "x",
            address: &x as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "y",
            address: &y as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
    ]
}

#[test]
fn test_integer_constant() {
    assert_eval("42", 42.0, &[]);
}

#[test]
fn test_decimal_constant() {
    assert_eval("3.14159", 3.14159, &[]);
}

#[test]
fn test_addition() {
    assert_eval("2+3", 5.0, &[]);
}

#[test]
fn test_subtraction() {
    assert_eval("10-4", 6.0, &[]);
}

#[test]
fn test_multiplication() {
    assert_eval("6*7", 42.0, &[]);
}

#[test]
fn test_division() {
    assert_eval("8/2", 4.0, &[]);
}

#[test]
fn test_modulus() {
    assert_eval("8%3", 2.0, &[]);
}

#[test]
fn test_power_operator() {
    assert_eval("2^3", 8.0, &[]);
}

#[test]
fn test_operator_precedence() {
    assert_eval("2+3*4", 14.0, &[]);
    assert_eval("(2+3)*4", 20.0, &[]);
}

#[test]
fn test_parentheses() {
    assert_eval("(1+2)*3", 9.0, &[]);
}

#[test]
fn test_nested_parentheses() {
    assert_eval("((2+3)*2)", 10.0, &[]);
}

#[test]
fn test_unary_plus() {
    assert_eval("+5", 5.0, &[]);
}

#[test]
fn test_unary_minus() {
    assert_eval("-5", -5.0, &[]);
}

#[test]
fn test_variable_lookup() {
    let lookup = variable_lookup(3.0, 4.0);
    assert_eval("x", 3.0, &lookup);
}

#[test]
fn test_multiple_variables() {
    let lookup = variable_lookup(2.0, 3.0);
    assert_eval("x+y", 5.0, &lookup);
}

#[test]
fn test_builtin_sqrt() {
    assert_eval("sqrt(9)", 3.0, &[]);
}

#[test]
fn test_builtin_sin() {
    assert_eval("sin(0)", 0.0, &[]);
}

#[test]
fn test_builtin_cos() {
    assert_eval("cos(0)", 1.0, &[]);
}

#[test]
fn test_builtin_pow() {
    assert_eval("pow(2,3)", 8.0, &[]);
}

#[test]
fn test_nested_functions() {
    assert_eval("sqrt(pow(2,2))", 2.0, &[]);
}

#[test]
fn test_factorial() {
    assert_eval("fac(5)", 120.0, &[]);
}

#[test]
fn test_ncr() {
    assert_eval("ncr(5,2)", 10.0, &[]);
}

#[test]
fn test_npr() {
    assert_eval("npr(5,2)", 20.0, &[]);
}

#[test]
fn test_invalid_expression() {
    assert_error("1+", &[]);
}

#[test]
fn test_unmatched_parentheses() {
    assert_error("(1+2", &[]);
}

#[test]
fn test_unknown_identifier() {
    assert_error("foo", &[]);
}

#[test]
fn test_empty_expression() {
    assert_error("", &[]);
}