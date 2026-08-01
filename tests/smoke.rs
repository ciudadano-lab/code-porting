//! Direct translation of `smoke.c`'s test suite. Every case here is the
//! same expression string and expected result as the original test file;
//! only the harness syntax changed.

use std::os::raw::c_void;
use tinyexpr::{te_compile, te_interp, Variable};

const TOL: f64 = 0.001; // LTEST_FLOAT_TOLERANCE in minctest.h

fn assert_feq(a: f64, b: f64, expr: &str) {
    let diff = (a - b).abs();
    assert!(diff <= TOL && diff == diff, "{expr}: {a} != {b}");
}

#[test]
fn test_results() {
    let cases: &[(&str, f64)] = &[
        ("1", 1.0),
        ("1 ", 1.0),
        ("(1)", 1.0),
        ("pi", 3.14159),
        ("atan(1)*4 - pi", 0.0),
        ("e", 2.71828),
        ("2+1", 3.0),
        ("(((2+(1))))", 3.0),
        ("3+2", 5.0),
        ("3+2+4", 9.0),
        ("(3+2)+4", 9.0),
        ("3+(2+4)", 9.0),
        ("(3+2+4)", 9.0),
        ("3*2*4", 24.0),
        ("(3*2)*4", 24.0),
        ("3*(2*4)", 24.0),
        ("(3*2*4)", 24.0),
        ("3-2-4", -3.0),
        ("(3-2)-4", -3.0),
        ("3-(2-4)", 5.0),
        ("(3-2-4)", -3.0),
        ("3/2/4", 3.0 / 2.0 / 4.0),
        ("(3/2)/4", (3.0 / 2.0) / 4.0),
        ("3/(2/4)", 3.0 / (2.0 / 4.0)),
        ("(3/2/4)", 3.0 / 2.0 / 4.0),
        ("(3*2/4)", 3.0 * 2.0 / 4.0),
        ("(3/2*4)", 3.0 / 2.0 * 4.0),
        ("3*(2/4)", 3.0 * (2.0 / 4.0)),
        ("asin sin .5", 0.5),
        ("sin asin .5", 0.5),
        ("ln exp .5", 0.5),
        ("exp ln .5", 0.5),
        ("asin sin-.5", -0.5),
        ("asin sin-0.5", -0.5),
        ("asin sin -0.5", -0.5),
        ("asin (sin -0.5)", -0.5),
        ("asin (sin (-0.5))", -0.5),
        ("asin sin (-0.5)", -0.5),
        ("(asin sin (-0.5))", -0.5),
        ("log10 1000", 3.0),
        ("log10 1e3", 3.0),
        ("log10(1000)", 3.0),
        ("log10(1e3)", 3.0),
        ("log10 1.0e3", 3.0),
        ("10^5*5e-5", 5.0),
        // TE_NAT_LOG is off by default in the shipped C source, so `log`
        // means base-10 log -- see builtins::NAT_LOG.
        ("log 1000", 3.0),
        ("ln (e^10)", 10.0),
        ("100^.5+1", 11.0),
        ("100 ^.5+1", 11.0),
        ("100^+.5+1", 11.0),
        ("100^--.5+1", 11.0),
        ("100^---+-++---++-+-+-.5+1", 11.0),
        ("100^-.5+1", 1.1),
        ("100^---.5+1", 1.1),
        ("100^+---.5+1", 1.1),
        ("1e2^+---.5e0+1e0", 1.1),
        ("--(1e2^(+(-(-(-.5e0))))+1e0)", 1.1),
        ("sqrt 100 + 7", 17.0),
        ("sqrt 100 * 7", 70.0),
        ("sqrt (100 * 100)", 100.0),
        ("1,2", 2.0),
        ("1,2+1", 3.0),
        ("1+1,2+2,2+1", 3.0),
        ("1,2,3", 3.0),
        ("(1,2),3", 3.0),
        ("1,(2,3)", 3.0),
        ("-(1,(2,3))", -3.0),
        ("2^2", 4.0),
        ("pow(2,2)", 4.0),
        ("atan2(1,1)", 0.7854),
        ("atan2(1,2)", 0.4636),
        ("atan2(2,1)", 1.1071),
        ("atan2(3,4)", 0.6435),
        ("atan2(3+3,4*2)", 0.6435),
        ("atan2(3+3,(4*2))", 0.6435),
        ("atan2((3+3),4*2)", 0.6435),
        ("atan2((3+3),(4*2))", 0.6435),
    ];

    for (expr, answer) in cases {
        let ev = te_interp(expr).unwrap_or_else(|e| panic!("{expr}: unexpected error at {e}"));
        assert_feq(ev, *answer, expr);
    }
}

#[test]
fn test_syntax() {
    let errors: &[(&str, usize)] = &[
        ("", 1),
        ("1+", 2),
        ("1)", 2),
        ("(1", 2),
        ("1**1", 3),
        ("1*2(+4", 4),
        ("1*2(1+4", 4),
        ("a+5", 1),
        ("!+5", 1),
        ("_a+5", 1),
        ("#a+5", 1),
        ("1^^5", 3),
        ("1**5", 3),
        ("sin(cos5", 8),
    ];

    for (expr, e) in errors {
        match te_interp(expr) {
            Ok(v) => panic!("{expr}: expected error {e}, got Ok({v})"),
            Err(pos) => assert_eq!(pos, *e, "{expr}"),
        }
        match te_compile(expr, &[]) {
            Ok(_) => panic!("{expr}: expected compile error {e}"),
            Err(pos) => assert_eq!(pos, *e, "{expr}"),
        }
    }
}

#[test]
fn test_nans() {
    let nans = [
        "0/0",
        "1%0",
        "1%(1%0)",
        "(1%0)%1",
        "fac(-1)",
        "ncr(2, 4)",
        "ncr(-2, 4)",
        "ncr(2, -4)",
        "npr(2, 4)",
        "npr(-2, 4)",
        "npr(2, -4)",
    ];
    for expr in nans {
        let r = te_interp(expr).expect("should compile without error");
        assert!(r.is_nan(), "{expr}: expected NaN, got {r}");

        let n = te_compile(expr, &[]).expect("should compile");
        let c = n.eval();
        assert!(c.is_nan(), "{expr}: expected NaN, got {c}");
    }
}

#[test]
fn test_infs() {
    let infs = [
        "1/0",
        "log(0)",
        "pow(2,10000000)",
        "fac(300)",
        "ncr(300,100)",
        "ncr(300000,100)",
        "ncr(300000,100)*8",
        "npr(3,2)*ncr(300000,100)",
        "npr(100,90)",
        "npr(30,25)",
    ];
    for expr in infs {
        let r = te_interp(expr).expect("should compile without error");
        assert!(r.is_infinite(), "{expr}: expected inf, got {r}");

        let n = te_compile(expr, &[]).expect("should compile");
        let c = n.eval();
        assert!(c.is_infinite(), "{expr}: expected inf, got {c}");
    }
}

#[test]
fn test_variables() {
    let mut x = 0.0f64;
    let mut y = 0.0f64;
    let mut test = 0.0f64;
    let lookup2 = [Variable::variable("x", &x), Variable::variable("y", &y)];
    let lookup3 = [
        Variable::variable("x", &x),
        Variable::variable("y", &y),
        Variable::variable("te_st", &test),
    ];

    let expr1 = te_compile("cos x + sin y", &lookup2).unwrap();
    let expr2 = te_compile("x+x+x-y", &lookup2).unwrap();
    let expr3 = te_compile("x*y^3", &lookup2).unwrap();
    let expr4 = te_compile("te_st+5", &lookup3).unwrap();

    y = 2.0;
    while y < 3.0 {
        x = 0.0;
        while x < 5.0 {
            assert_feq(expr1.eval(), x.cos() + y.sin(), "expr1");
            assert_feq(expr2.eval(), x + x + x - y, "expr2");
            assert_feq(expr3.eval(), x * y * y * y, "expr3");
            #[allow(unused_assignments)]
            {
                test = x;
            }
            assert_feq(expr4.eval(), x + 5.0, "expr4");
            x += 1.0;
        }
        y += 1.0;
    }

    assert!(te_compile("xx*y^3", &lookup2).is_err());
    assert!(te_compile("tes", &lookup3).is_err());
    assert!(te_compile("sinn x", &lookup2).is_err());
    assert!(te_compile("si x", &lookup2).is_err());
}

#[test]
fn test_functions() {
    let mut x = -5.0f64;
    let mut y = 0.0f64;
    let lookup = [Variable::variable("x", &x), Variable::variable("y", &y)];

    macro_rules! cross_check {
        ($e:expr, $b:expr) => {{
            let b = $b;
            if !b.is_nan() {
                let expr = te_compile($e, &lookup).unwrap_or_else(|e| panic!("{}: {}", $e, e));
                assert_feq(expr.eval(), b, $e);
            }
        }};
    }

    while x < 5.0 {
        cross_check!("abs x", x.abs());
        cross_check!("acos x", x.acos());
        cross_check!("asin x", x.asin());
        cross_check!("atan x", x.atan());
        cross_check!("ceil x", x.ceil());
        cross_check!("cos x", x.cos());
        cross_check!("cosh x", x.cosh());
        cross_check!("exp x", x.exp());
        cross_check!("floor x", x.floor());
        cross_check!("ln x", x.ln());
        cross_check!("log10 x", x.log10());
        cross_check!("sin x", x.sin());
        cross_check!("sinh x", x.sinh());
        cross_check!("sqrt x", x.sqrt());
        cross_check!("tan x", x.tan());
        cross_check!("tanh x", x.tanh());

        y = -2.0;
        while y < 2.0 {
            if x.abs() < 0.01 {
                break;
            }
            cross_check!("atan2(x,y)", x.atan2(y));
            cross_check!("pow(x,y)", x.powf(y));
            y += 0.2;
        }
        x += 0.2;
    }
}

fn sum0() -> f64 {
    6.0
}
fn sum1(a: f64) -> f64 {
    a * 2.0
}
fn sum2(a: f64, b: f64) -> f64 {
    a + b
}
fn sum3(a: f64, b: f64, c: f64) -> f64 {
    a + b + c
}
fn sum4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a + b + c + d
}
fn sum5(a: f64, b: f64, c: f64, d: f64, e: f64) -> f64 {
    a + b + c + d + e
}
fn sum6(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> f64 {
    a + b + c + d + e + f
}
fn sum7(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64) -> f64 {
    a + b + c + d + e + f + g
}

#[test]
fn test_dynamic() {
    let x = 2.0f64;
    let f = 5.0f64;
    let lookup = [
        Variable::variable("x", &x),
        Variable::variable("f", &f),
        Variable::function0("sum0", sum0, false),
        Variable::function1("sum1", sum1, false),
        Variable::function2("sum2", sum2, false),
        Variable::function3("sum3", sum3, false),
        Variable::function4("sum4", sum4, false),
        Variable::function5("sum5", sum5, false),
        Variable::function6("sum6", sum6, false),
        Variable::function7("sum7", sum7, false),
    ];

    let cases: &[(&str, f64)] = &[
        ("x", 2.0),
        ("f+x", 7.0),
        ("x+x", 4.0),
        ("x+f", 7.0),
        ("f+f", 10.0),
        ("f+sum0", 11.0),
        ("sum0+sum0", 12.0),
        ("sum0()+sum0", 12.0),
        ("sum0+sum0()", 12.0),
        ("sum0()+(0)+sum0()", 12.0),
        ("sum1 sum0", 12.0),
        ("sum1(sum0)", 12.0),
        ("sum1 f", 10.0),
        ("sum1 x", 4.0),
        ("sum2 (sum0, x)", 8.0),
        ("sum3 (sum0, x, 2)", 10.0),
        ("sum2(2,3)", 5.0),
        ("sum3(2,3,4)", 9.0),
        ("sum4(2,3,4,5)", 14.0),
        ("sum5(2,3,4,5,6)", 20.0),
        ("sum6(2,3,4,5,6,7)", 27.0),
        ("sum7(2,3,4,5,6,7,8)", 35.0),
    ];

    for (expr, answer) in cases {
        let ex = te_compile(expr, &lookup).unwrap_or_else(|e| panic!("{expr}: error {e}"));
        assert_feq(ex.eval(), *answer, expr);
    }
}

fn clo0(context: *mut c_void) -> f64 {
    if !context.is_null() {
        unsafe { *(context as *const f64) + 6.0 }
    } else {
        6.0
    }
}
fn clo1(context: *mut c_void, a: f64) -> f64 {
    if !context.is_null() {
        unsafe { *(context as *const f64) + a * 2.0 }
    } else {
        a * 2.0
    }
}
fn clo2(context: *mut c_void, a: f64, b: f64) -> f64 {
    if !context.is_null() {
        unsafe { *(context as *const f64) + a + b }
    } else {
        a + b
    }
}
fn cell(context: *mut c_void, a: f64) -> f64 {
    unsafe {
        let c = context as *const f64;
        *c.add(a as usize)
    }
}

#[test]
fn test_closure() {
    let mut extra = 0.0f64;
    let c = [5.0f64, 6.0, 7.0, 8.0, 9.0];

    let extra_ptr = &mut extra as *mut f64 as *mut c_void;
    let c_ptr = c.as_ptr() as *mut c_void;

    let lookup = [
        Variable::closure0("c0", clo0 as fn(*mut c_void) -> f64, false, extra_ptr),
        Variable::closure1("c1", clo1 as fn(*mut c_void, f64) -> f64, false, extra_ptr),
        Variable::closure2("c2", clo2 as fn(*mut c_void, f64, f64) -> f64, false, extra_ptr),
        Variable::closure1("cell", cell as fn(*mut c_void, f64) -> f64, false, c_ptr),
    ];

    let cases: &[(&str, f64)] = &[("c0", 6.0), ("c1 4", 8.0), ("c2 (10, 20)", 30.0)];

    for (expr, answer) in cases {
        let ex = te_compile(expr, &lookup).unwrap_or_else(|e| panic!("{expr}: error {e}"));

        extra = 0.0;
        assert_feq(ex.eval(), answer + extra, expr);

        extra = 10.0;
        assert_feq(ex.eval(), answer + extra, expr);
    }

    let cases2: &[(&str, f64)] = &[
        ("cell 0", 5.0),
        ("cell 1", 6.0),
        ("cell 0 + cell 1", 11.0),
        ("cell 1 * cell 3 + cell 4", 57.0),
    ];
    for (expr, answer) in cases2 {
        let ex = te_compile(expr, &lookup).unwrap_or_else(|e| panic!("{expr}: error {e}"));
        assert_feq(ex.eval(), *answer, expr);
    }
}

#[test]
fn test_optimize() {
    let cases: &[(&str, f64)] = &[("5+5", 10.0), ("pow(2,2)", 4.0), ("sqrt 100", 10.0), ("pi * 2", 6.2832)];
    for (expr, answer) in cases {
        let ex = te_compile(expr, &[]).unwrap_or_else(|e| panic!("{expr}: error {e}"));
        // The answer should be known without even running eval -- i.e. the
        // tree must already have been folded into a single constant node.
        assert!(ex.is_constant(), "{expr}: not folded to a constant");
        assert_feq(ex.eval(), *answer, expr);
    }
}

#[test]
fn test_pow() {
    // Matches the #else (non-TE_POW_FROM_RIGHT) branch of test_pow in
    // smoke.c, since that macro is left undefined in the shipped source
    // and in this port.
    let cases: &[(&str, &str)] = &[
        ("2^3^4", "(2^3)^4"),
        ("-2^2", "(-2)^2"),
        ("(-2)^2", "4"),
        ("--2^2", "2^2"),
        ("---2^2", "(-2)^2"),
        ("-2^2", "4"),
        ("2^1.1^1.2^1.3", "((2^1.1)^1.2)^1.3"),
        ("-a^b", "(-a)^b"),
        ("-a^-b", "(-a)^(-b)"),
        ("1^0", "1"),
        ("(1)^0", "1"),
        ("(-1)^0", "1"),
        ("(-5)^0", "1"),
        ("-2^-3^-4", "((-2)^(-3))^(-4)"),
    ];

    let a = 2.0f64;
    let b = 3.0f64;
    let lookup = [Variable::variable("a", &a), Variable::variable("b", &b)];

    for (e1, e2) in cases {
        let ex1 = te_compile(e1, &lookup).unwrap_or_else(|e| panic!("{e1}: error {e}"));
        let ex2 = te_compile(e2, &lookup).unwrap_or_else(|e| panic!("{e2}: error {e}"));
        assert_feq(ex1.eval(), ex2.eval(), &format!("{e1} <> {e2}"));
    }
}

#[test]
fn test_combinatorics() {
    let cases: &[(&str, f64)] = &[
        ("fac(0)", 1.0),
        ("fac(0.2)", 1.0),
        ("fac(1)", 1.0),
        ("fac(2)", 2.0),
        ("fac(3)", 6.0),
        ("fac(4.8)", 24.0),
        ("fac(10)", 3628800.0),
        ("ncr(0,0)", 1.0),
        ("ncr(10,1)", 10.0),
        ("ncr(10,0)", 1.0),
        ("ncr(10,10)", 1.0),
        ("ncr(16,7)", 11440.0),
        ("ncr(16,9)", 11440.0),
        ("ncr(100,95)", 75287520.0),
        ("npr(0,0)", 1.0),
        ("npr(10,1)", 10.0),
        ("npr(10,0)", 1.0),
        ("npr(10,10)", 3628800.0),
        ("npr(20,5)", 1860480.0),
        ("npr(100,4)", 94109400.0),
    ];
    for (expr, answer) in cases {
        let ev = te_interp(expr).unwrap_or_else(|e| panic!("{expr}: error {e}"));
        assert_feq(ev, *answer, expr);
    }
}
