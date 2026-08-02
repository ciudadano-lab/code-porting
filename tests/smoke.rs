/*
 * TINYEXPR - Tiny recursive descent parser and evaluation engine in C
 *
 * Copyright (c) 2015-2020 Lewis Van Winkle
 *
 * http://CodePlea.com
 *
 * This software is provided 'as-is', without any express or implied
 * warranty. In no event will the authors be held liable for any damages
 * arising from the use of this software.
 *
 * Permission is granted to anyone to use this software for any purpose,
 * including commercial applications, and to alter it and redistribute it
 * freely, subject to the following restrictions:
 *
 * 1. The origin of this software must not be misrepresented; you must not
 * claim that you wrote the original software. If you use this software
 * in a product, an acknowledgement in the product documentation would be
 * appreciated but is not required.
 * 2. Altered source versions must be plainly marked as such, and must not be
 * misrepresented as being the original software.
 * 3. This notice may not be removed or altered from any source distribution.
 */

use tinyexpr::{
    te_compile, te_eval_expr, te_interp, TeVariable,
    TE_VARIABLE, TE_FUNCTION0, TE_FUNCTION1, TE_FUNCTION2, TE_FUNCTION3,
    TE_FUNCTION4, TE_FUNCTION5, TE_FUNCTION6, TE_FUNCTION7,
    TE_CLOSURE0, TE_CLOSURE1, TE_CLOSURE2,
};
use std::os::raw::c_void;
use std::ptr;

// ---------------------------------------------------------------------------
// Minimal minctest-compatible helpers (same behaviour as original macros)
// ---------------------------------------------------------------------------
static mut LTESTS: i32 = 0;
static mut LFAILS: i32 = 0;

macro_rules! lok {
    ($cond:expr) => {{
        unsafe {
            LTESTS += 1;
            if !($cond) {
                LFAILS += 1;
                println!("FAIL {}:{}  {}", file!(), line!(), stringify!($cond));
            }
        }
    }};
}

macro_rules! lequal {
    ($a:expr, $b:expr) => {{
        unsafe {
            LTESTS += 1;
            if ($a) != ($b) {
                LFAILS += 1;
                println!("FAIL {}:{}  {} == {}", file!(), line!(), $a, $b);
            }
        }
    }};
}

macro_rules! lfequal {
    ($a:expr, $b:expr) => {{
        unsafe {
            LTESTS += 1;
            let a = $a as f64;
            let b = $b as f64;
            // same tolerance spirit as original minctest lfequal
            if (a - b).abs() > 1e-4 && !(a != a && b != b) {
                LFAILS += 1;
                println!("FAIL {}:{}  {} ~= {}", file!(), line!(), a, b);
            }
        }
    }};
}

fn lrun(name: &str, test: fn()) {
    let fails_before = unsafe { LFAILS };
    test();
    let fails_after = unsafe { LFAILS };
    if fails_after == fails_before {
        println!("PASS  {}", name);
    } else {
        println!("FAIL  {}", name);
    }
}

fn lresults() {
    unsafe {
        println!();
        println!("{} tests, {} failures", LTESTS, LFAILS);
    }
}

// ---------------------------------------------------------------------------
// Test data structures (identical)
// ---------------------------------------------------------------------------
struct TestCase {
    expr: &'static str,
    answer: f64,
}

struct TestEqu {
    expr1: &'static str,
    expr2: &'static str,
}

// ---------------------------------------------------------------------------
// test_results
// ---------------------------------------------------------------------------
fn test_results() {
    let cases = [
        TestCase { expr: "1", answer: 1.0 },
        TestCase { expr: "1 ", answer: 1.0 },
        TestCase { expr: "(1)", answer: 1.0 },
        TestCase { expr: "pi", answer: 3.14159 },
        TestCase { expr: "atan(1)*4 - pi", answer: 0.0 },
        TestCase { expr: "e", answer: 2.71828 },
        TestCase { expr: "2+1", answer: 2.0 + 1.0 },
        TestCase { expr: "(((2+(1))))", answer: 2.0 + 1.0 },
        TestCase { expr: "3+2", answer: 3.0 + 2.0 },
        TestCase { expr: "3+2+4", answer: 3.0 + 2.0 + 4.0 },
        TestCase { expr: "(3+2)+4", answer: 3.0 + 2.0 + 4.0 },
        TestCase { expr: "3+(2+4)", answer: 3.0 + 2.0 + 4.0 },
        TestCase { expr: "(3+2+4)", answer: 3.0 + 2.0 + 4.0 },
        TestCase { expr: "3*2*4", answer: 3.0 * 2.0 * 4.0 },
        TestCase { expr: "(3*2)*4", answer: 3.0 * 2.0 * 4.0 },
        TestCase { expr: "3*(2*4)", answer: 3.0 * 2.0 * 4.0 },
        TestCase { expr: "(3*2*4)", answer: 3.0 * 2.0 * 4.0 },
        TestCase { expr: "3-2-4", answer: 3.0 - 2.0 - 4.0 },
        TestCase { expr: "(3-2)-4", answer: (3.0 - 2.0) - 4.0 },
        TestCase { expr: "3-(2-4)", answer: 3.0 - (2.0 - 4.0) },
        TestCase { expr: "(3-2-4)", answer: 3.0 - 2.0 - 4.0 },
        TestCase { expr: "3/2/4", answer: 3.0 / 2.0 / 4.0 },
        TestCase { expr: "(3/2)/4", answer: (3.0 / 2.0) / 4.0 },
        TestCase { expr: "3/(2/4)", answer: 3.0 / (2.0 / 4.0) },
        TestCase { expr: "(3/2/4)", answer: 3.0 / 2.0 / 4.0 },
        TestCase { expr: "(3*2/4)", answer: 3.0 * 2.0 / 4.0 },
        TestCase { expr: "(3/2*4)", answer: 3.0 / 2.0 * 4.0 },
        TestCase { expr: "3*(2/4)", answer: 3.0 * (2.0 / 4.0) },
        TestCase { expr: "asin sin .5", answer: 0.5 },
        TestCase { expr: "sin asin .5", answer: 0.5 },
        TestCase { expr: "ln exp .5", answer: 0.5 },
        TestCase { expr: "exp ln .5", answer: 0.5 },
        TestCase { expr: "asin sin-.5", answer: -0.5 },
        TestCase { expr: "asin sin-0.5", answer: -0.5 },
        TestCase { expr: "asin sin -0.5", answer: -0.5 },
        TestCase { expr: "asin (sin -0.5)", answer: -0.5 },
        TestCase { expr: "asin (sin (-0.5))", answer: -0.5 },
        TestCase { expr: "asin sin (-0.5)", answer: -0.5 },
        TestCase { expr: "(asin sin (-0.5))", answer: -0.5 },
        TestCase { expr: "log10 1000", answer: 3.0 },
        TestCase { expr: "log10 1e3", answer: 3.0 },
        TestCase { expr: "log10 1000", answer: 3.0 },
        TestCase { expr: "log10 1e3", answer: 3.0 },
        TestCase { expr: "log10(1000)", answer: 3.0 },
        TestCase { expr: "log10(1e3)", answer: 3.0 },
        TestCase { expr: "log10 1.0e3", answer: 3.0 },
        TestCase { expr: "10^5*5e-5", answer: 5.0 },
        // #ifdef TE_NAT_LOG
        // {"log 1000", 6.9078},
        // {"log e", 1},
        // {"log (e^10)", 10},
        // #else
        TestCase { expr: "log 1000", answer: 3.0 },
        // #endif
        TestCase { expr: "ln (e^10)", answer: 10.0 },
        TestCase { expr: "100^.5+1", answer: 11.0 },
        TestCase { expr: "100 ^.5+1", answer: 11.0 },
        TestCase { expr: "100^+.5+1", answer: 11.0 },
        TestCase { expr: "100^--.5+1", answer: 11.0 },
        TestCase { expr: "100^---+-++---++-+-+-.5+1", answer: 11.0 },
        TestCase { expr: "100^-.5+1", answer: 1.1 },
        TestCase { expr: "100^---.5+1", answer: 1.1 },
        TestCase { expr: "100^+---.5+1", answer: 1.1 },
        TestCase { expr: "1e2^+---.5e0+1e0", answer: 1.1 },
        TestCase { expr: "--(1e2^(+(-(-(-.5e0))))+1e0)", answer: 1.1 },
        TestCase { expr: "sqrt 100 + 7", answer: 17.0 },
        TestCase { expr: "sqrt 100 * 7", answer: 70.0 },
        TestCase { expr: "sqrt (100 * 100)", answer: 100.0 },
        TestCase { expr: "1,2", answer: 2.0 },
        TestCase { expr: "1,2+1", answer: 3.0 },
        TestCase { expr: "1+1,2+2,2+1", answer: 3.0 },
        TestCase { expr: "1,2,3", answer: 3.0 },
        TestCase { expr: "(1,2),3", answer: 3.0 },
        TestCase { expr: "1,(2,3)", answer: 3.0 },
        TestCase { expr: "-(1,(2,3))", answer: -3.0 },
        TestCase { expr: "2^2", answer: 4.0 },
        TestCase { expr: "pow(2,2)", answer: 4.0 },
        TestCase { expr: "atan2(1,1)", answer: 0.7854 },
        TestCase { expr: "atan2(1,2)", answer: 0.4636 },
        TestCase { expr: "atan2(2,1)", answer: 1.1071 },
        TestCase { expr: "atan2(3,4)", answer: 0.6435 },
        TestCase { expr: "atan2(3+3,4*2)", answer: 0.6435 },
        TestCase { expr: "atan2(3+3,(4*2))", answer: 0.6435 },
        TestCase { expr: "atan2((3+3),4*2)", answer: 0.6435 },
        TestCase { expr: "atan2((3+3),(4*2))", answer: 0.6435 },
    ];

    for c in &cases {
        match te_interp(c.expr) {
            Ok(ev) => {
                lok!(true); // err == 0
                lfequal!(ev, c.answer);
            }
            Err(err) => {
                lok!(false);
                println!("FAILED: {} ({})", c.expr, err);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// test_syntax
// ---------------------------------------------------------------------------
fn test_syntax() {
    let errors = [
        TestCase { expr: "", answer: 1.0 },
        TestCase { expr: "1+", answer: 2.0 },
        TestCase { expr: "1)", answer: 2.0 },
        TestCase { expr: "(1", answer: 2.0 },
        TestCase { expr: "1**1", answer: 3.0 },
        TestCase { expr: "1*2(+4", answer: 4.0 },
        TestCase { expr: "1*2(1+4", answer: 4.0 },
        TestCase { expr: "a+5", answer: 1.0 },
        TestCase { expr: "!+5", answer: 1.0 },
        TestCase { expr: "_a+5", answer: 1.0 },
        TestCase { expr: "#a+5", answer: 1.0 },
        TestCase { expr: "1^^5", answer: 3.0 },
        TestCase { expr: "1**5", answer: 3.0 },
        TestCase { expr: "sin(cos5", answer: 8.0 },
    ];

    for c in &errors {
        let e = c.answer as i32;

        match te_interp(c.expr) {
            Ok(r) => {
                lequal!(0, e); // should have failed
                lok!(r != r);
            }
            Err(err) => {
                lequal!(err, e);
                lok!(true); // r would be NaN
            }
        }

        match te_compile(c.expr, &[]) {
            Ok(_) => {
                lequal!(0, e);
                lok!(false);
            }
            Err(err) => {
                lequal!(err, e);
                lok!(true);
            }
        }

        if let Ok(k) = te_interp(c.expr) {
            lok!(k != k);
        } else {
            lok!(true);
        }
    }
}

// ---------------------------------------------------------------------------
// test_nans
// ---------------------------------------------------------------------------
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

    for expr in &nans {
        match te_interp(expr) {
            Ok(r) => {
                lequal!(0, 0); // err == 0
                lok!(r != r);
            }
            Err(_) => {
                lok!(false);
            }
        }

        match te_compile(expr, &[]) {
            Ok(n) => {
                lok!(true);
                lequal!(0, 0);
                let c = te_eval_expr(&n);
                lok!(c != c);
            }
            Err(_) => {
                lok!(false);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// test_infs
// ---------------------------------------------------------------------------
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

    for expr in &infs {
        match te_interp(expr) {
            Ok(r) => {
                lequal!(0, 0);
                lok!(r == r + 1.0);
            }
            Err(_) => {
                lok!(false);
            }
        }

        match te_compile(expr, &[]) {
            Ok(n) => {
                lok!(true);
                lequal!(0, 0);
                let c = te_eval_expr(&n);
                lok!(c == c + 1.0);
            }
            Err(_) => {
                lok!(false);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// test_variables
// ---------------------------------------------------------------------------
fn test_variables() {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut test: f64 = 0.0;

    let lookup = [
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
        TeVariable {
            name: "te_st",
            address: &test as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
    ];

    let expr1 = te_compile("cos x + sin y", &lookup[..2]).unwrap();
    lok!(true);
    let expr2 = te_compile("x+x+x-y", &lookup[..2]).unwrap();
    lok!(true);
    let expr3 = te_compile("x*y^3", &lookup[..2]).unwrap();
    lok!(true);
    let expr4 = te_compile("te_st+5", &lookup).unwrap();
    lok!(true);

    y = 2.0;
    while y < 3.0 {
        x = 0.0;
        while x < 5.0 {
            let ev = te_eval_expr(&expr1);
            lfequal!(ev, x.cos() + y.sin());

            let ev = te_eval_expr(&expr2);
            lfequal!(ev, x + x + x - y);

            let ev = te_eval_expr(&expr3);
            lfequal!(ev, x * y * y * y);

            let _ = x;
            let ev = te_eval_expr(&expr4);
            lfequal!(ev, x + 5.0);

            x += 1.0;
        }
        y += 1.0;
    }

    // error cases
    lok!(te_compile("xx*y^3", &lookup[..2]).is_err());
    lok!(te_compile("tes", &lookup).is_err());
    lok!(te_compile("sinn x", &lookup[..2]).is_err());
    lok!(te_compile("si x", &lookup[..2]).is_err());
}

// ---------------------------------------------------------------------------
// test_functions
// ---------------------------------------------------------------------------
fn test_functions() {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;

    let lookup = [
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
    ];

    x = -5.0;
    while x < 5.0 {
        // cross_check macro expanded
        macro_rules! cross_check {
            ($a:expr, $b:expr) => {{
                if ($b) == ($b) {
                    if let Ok(expr) = te_compile($a, &lookup) {
                        lfequal!(te_eval_expr(&expr), $b);
                        lok!(true);
                    }
                }
            }};
        }

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

// ---------------------------------------------------------------------------
// Dynamic functions
// ---------------------------------------------------------------------------
unsafe fn sum0() -> f64 {
    6.0
}
unsafe fn sum1(a: f64) -> f64 {
    a * 2.0
}
unsafe fn sum2(a: f64, b: f64) -> f64 {
    a + b
}
unsafe fn sum3(a: f64, b: f64, c: f64) -> f64 {
    a + b + c
}
unsafe fn sum4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a + b + c + d
}
unsafe fn sum5(a: f64, b: f64, c: f64, d: f64, e: f64) -> f64 {
    a + b + c + d + e
}
unsafe fn sum6(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> f64 {
    a + b + c + d + e + f
}
unsafe fn sum7(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64) -> f64 {
    a + b + c + d + e + f + g
}

fn test_dynamic() {
    let mut x: f64 = 2.0;
    let mut f: f64 = 5.0;

    let lookup = [
        TeVariable {
            name: "x",
            address: &x as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "f",
            address: &f as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum0",
            address: sum0 as *const c_void,
            type_: TE_FUNCTION0,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum1",
            address: sum1 as *const c_void,
            type_: TE_FUNCTION1,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum2",
            address: sum2 as *const c_void,
            type_: TE_FUNCTION2,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum3",
            address: sum3 as *const c_void,
            type_: TE_FUNCTION3,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum4",
            address: sum4 as *const c_void,
            type_: TE_FUNCTION4,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum5",
            address: sum5 as *const c_void,
            type_: TE_FUNCTION5,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum6",
            address: sum6 as *const c_void,
            type_: TE_FUNCTION6,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "sum7",
            address: sum7 as *const c_void,
            type_: TE_FUNCTION7,
            context: ptr::null_mut(),
        },
    ];

    let cases = [
        TestCase { expr: "x", answer: 2.0 },
        TestCase { expr: "f+x", answer: 7.0 },
        TestCase { expr: "x+x", answer: 4.0 },
        TestCase { expr: "x+f", answer: 7.0 },
        TestCase { expr: "f+f", answer: 10.0 },
        TestCase { expr: "f+sum0", answer: 11.0 },
        TestCase { expr: "sum0+sum0", answer: 12.0 },
        TestCase { expr: "sum0()+sum0", answer: 12.0 },
        TestCase { expr: "sum0+sum0()", answer: 12.0 },
        TestCase { expr: "sum0()+(0)+sum0()", answer: 12.0 },
        TestCase { expr: "sum1 sum0", answer: 12.0 },
        TestCase { expr: "sum1(sum0)", answer: 12.0 },
        TestCase { expr: "sum1 f", answer: 10.0 },
        TestCase { expr: "sum1 x", answer: 4.0 },
        TestCase { expr: "sum2 (sum0, x)", answer: 8.0 },
        TestCase { expr: "sum3 (sum0, x, 2)", answer: 10.0 },
        TestCase { expr: "sum2(2,3)", answer: 5.0 },
        TestCase { expr: "sum3(2,3,4)", answer: 9.0 },
        TestCase { expr: "sum4(2,3,4,5)", answer: 14.0 },
        TestCase { expr: "sum5(2,3,4,5,6)", answer: 20.0 },
        TestCase { expr: "sum6(2,3,4,5,6,7)", answer: 27.0 },
        TestCase { expr: "sum7(2,3,4,5,6,7,8)", answer: 35.0 },
    ];

    for c in &cases {
        if let Ok(ex) = te_compile(c.expr, &lookup) {
            lok!(true);
            lfequal!(te_eval_expr(&ex), c.answer);
        } else {
            lok!(false);
        }
    }
}

// ---------------------------------------------------------------------------
// Closures
// ---------------------------------------------------------------------------
unsafe fn clo0(context: *mut c_void) -> f64 {
    if !context.is_null() {
        return *(context as *const f64) + 6.0;
    }
    6.0
}
unsafe fn clo1(context: *mut c_void, a: f64) -> f64 {
    if !context.is_null() {
        return *(context as *const f64) + a * 2.0;
    }
    a * 2.0
}
unsafe fn clo2(context: *mut c_void, a: f64, b: f64) -> f64 {
    if !context.is_null() {
        return *(context as *const f64) + a + b;
    }
    a + b
}
unsafe fn cell(context: *mut c_void, a: f64) -> f64 {
    let c = context as *const f64;
    *c.offset(a as isize)
}

fn test_closure() {
    let mut extra: f64 = 0.0;
    let c: [f64; 5] = [5.0, 6.0, 7.0, 8.0, 9.0];

    let lookup = [
        TeVariable {
            name: "c0",
            address: clo0 as *const c_void,
            type_: TE_CLOSURE0,
            context: &mut extra as *mut f64 as *mut c_void,
        },
        TeVariable {
            name: "c1",
            address: clo1 as *const c_void,
            type_: TE_CLOSURE1,
            context: &mut extra as *mut f64 as *mut c_void,
        },
        TeVariable {
            name: "c2",
            address: clo2 as *const c_void,
            type_: TE_CLOSURE2,
            context: &mut extra as *mut f64 as *mut c_void,
        },
        TeVariable {
            name: "cell",
            address: cell as *const c_void,
            type_: TE_CLOSURE1,
            context: c.as_ptr() as *mut c_void,
        },
    ];

    let cases = [
        TestCase { expr: "c0", answer: 6.0 },
        TestCase { expr: "c1 4", answer: 8.0 },
        TestCase { expr: "c2 (10, 20)", answer: 30.0 },
    ];

    for c in &cases {
        if let Ok(ex) = te_compile(c.expr, &lookup) {
            lok!(true);
            extra = 0.0;
            lfequal!(te_eval_expr(&ex), c.answer + extra);
            extra = 10.0;
            lfequal!(te_eval_expr(&ex), c.answer + extra);
        } else {
            lok!(false);
        }
    }

    let cases2 = [
        TestCase { expr: "cell 0", answer: 5.0 },
        TestCase { expr: "cell 1", answer: 6.0 },
        TestCase { expr: "cell 0 + cell 1", answer: 11.0 },
        TestCase { expr: "cell 1 * cell 3 + cell 4", answer: 57.0 },
    ];

    for c in &cases2 {
        if let Ok(ex) = te_compile(c.expr, &lookup) {
            lok!(true);
            lfequal!(te_eval_expr(&ex), c.answer);
        } else {
            lok!(false);
        }
    }
}

// ---------------------------------------------------------------------------
// test_optimize
// ---------------------------------------------------------------------------
fn test_optimize() {
    let cases = [
        TestCase { expr: "5+5", answer: 10.0 },
        TestCase { expr: "pow(2,2)", answer: 4.0 },
        TestCase { expr: "sqrt 100", answer: 10.0 },
        TestCase { expr: "pi * 2", answer: 6.2832 },
    ];

    for c in &cases {
        if let Ok(ex) = te_compile(c.expr, &[]) {
            lok!(true);
            // After optimize the constant should already be known
            lfequal!(ex.value, c.answer);
            lfequal!(te_eval_expr(&ex), c.answer);
        } else {
            lok!(false);
        }
    }
}

// ---------------------------------------------------------------------------
// test_pow (left-associative default)
// ---------------------------------------------------------------------------
fn test_pow() {
    // #else branch (TE_POW_FROM_RIGHT not defined)
    let cases = [
        TestEqu { expr1: "2^3^4", expr2: "(2^3)^4" },
        TestEqu { expr1: "-2^2", expr2: "(-2)^2" },
        TestEqu { expr1: "(-2)^2", expr2: "4" },
        TestEqu { expr1: "--2^2", expr2: "2^2" },
        TestEqu { expr1: "---2^2", expr2: "(-2)^2" },
        TestEqu { expr1: "-2^2", expr2: "4" },
        TestEqu { expr1: "2^1.1^1.2^1.3", expr2: "((2^1.1)^1.2)^1.3" },
        TestEqu { expr1: "-a^b", expr2: "(-a)^b" },
        TestEqu { expr1: "-a^-b", expr2: "(-a)^(-b)" },
        TestEqu { expr1: "1^0", expr2: "1" },
        TestEqu { expr1: "(1)^0", expr2: "1" },
        TestEqu { expr1: "(-1)^0", expr2: "1" },
        TestEqu { expr1: "(-5)^0", expr2: "1" },
        TestEqu { expr1: "-2^-3^-4", expr2: "((-2)^(-3))^(-4)" },
    ];

    let mut a: f64 = 2.0;
    let mut b: f64 = 3.0;

    let lookup = [
        TeVariable {
            name: "a",
            address: &a as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
        TeVariable {
            name: "b",
            address: &b as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: ptr::null_mut(),
        },
    ];

    for c in &cases {
        let ex1 = te_compile(c.expr1, &lookup).unwrap();
        let ex2 = te_compile(c.expr2, &lookup).unwrap();
        lok!(true);
        lok!(true);

        let r1 = te_eval_expr(&ex1);
        let r2 = te_eval_expr(&ex2);

        let olfail = unsafe { LFAILS };
        lfequal!(r1, r2);
        if unsafe { LFAILS } != olfail {
            println!("Failed expression: {} <> {}", c.expr1, c.expr2);
        }
    }
}

// ---------------------------------------------------------------------------
// test_combinatorics
// ---------------------------------------------------------------------------
fn test_combinatorics() {
    let cases = [
        TestCase { expr: "fac(0)", answer: 1.0 },
        TestCase { expr: "fac(0.2)", answer: 1.0 },
        TestCase { expr: "fac(1)", answer: 1.0 },
        TestCase { expr: "fac(2)", answer: 2.0 },
        TestCase { expr: "fac(3)", answer: 6.0 },
        TestCase { expr: "fac(4.8)", answer: 24.0 },
        TestCase { expr: "fac(10)", answer: 3628800.0 },
        TestCase { expr: "ncr(0,0)", answer: 1.0 },
        TestCase { expr: "ncr(10,1)", answer: 10.0 },
        TestCase { expr: "ncr(10,0)", answer: 1.0 },
        TestCase { expr: "ncr(10,10)", answer: 1.0 },
        TestCase { expr: "ncr(16,7)", answer: 11440.0 },
        TestCase { expr: "ncr(16,9)", answer: 11440.0 },
        TestCase { expr: "ncr(100,95)", answer: 75287520.0 },
        TestCase { expr: "npr(0,0)", answer: 1.0 },
        TestCase { expr: "npr(10,1)", answer: 10.0 },
        TestCase { expr: "npr(10,0)", answer: 1.0 },
        TestCase { expr: "npr(10,10)", answer: 3628800.0 },
        TestCase { expr: "npr(20,5)", answer: 1860480.0 },
        TestCase { expr: "npr(100,4)", answer: 94109400.0 },
    ];

    for c in &cases {
        match te_interp(c.expr) {
            Ok(ev) => {
                lok!(true);
                lfequal!(ev, c.answer);
            }
            Err(err) => {
                lok!(false);
                println!("FAILED: {} ({})", c.expr, err);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
fn main() {
    lrun("Results", test_results);
    lrun("Syntax", test_syntax);
    lrun("NaNs", test_nans);
    lrun("INFs", test_infs);
    lrun("Variables", test_variables);
    lrun("Functions", test_functions);
    lrun("Dynamic", test_dynamic);
    lrun("Closure", test_closure);
    lrun("Optimize", test_optimize);
    lrun("Pow", test_pow);
    lrun("Combinatorics", test_combinatorics);
    lresults();

    let fails = unsafe { LFAILS };
    std::process::exit(if fails != 0 { 1 } else { 0 });
}