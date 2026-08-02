/*
 * TINYEXPR - Tiny recursive descent parser and evaluation engine in C
 *
 * Copyright (c) 2015, 2016 Lewis Van Winkle
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

use tinyexpr::{te_compile, te_eval_expr, TeVariable, TE_VARIABLE};
use std::os::raw::c_void;
use std::time::Instant;

const LOOPS: i32 = 10000;

type Function1 = fn(f64) -> f64;

fn bench(expr: &str, func: Function1) {
    let mut d: f64;
    let mut tmp: f64 = 0.0;

    let lk = TeVariable {
        name: "a",
        address: &tmp as *const f64 as *const c_void,
        type_: TE_VARIABLE,
        context: std::ptr::null_mut(),
    };

    println!("Expression: {}", expr);

    print!("native ");
    let start = Instant::now();
    d = 0.0;
    for _j in 0..LOOPS {
        for i in 0..LOOPS {
            tmp = i as f64;
            d += func(tmp);
        }
    }
    let nelapsed = start.elapsed().as_millis() as i32;

    /*Million floats per second input.*/
    print!(" {:.5}", d);
    if nelapsed != 0 {
        println!(
            "\t{:5}ms\t{:5}mfps",
            nelapsed,
            LOOPS as i64 * LOOPS as i64 / nelapsed as i64 / 1000
        );
    } else {
        println!("\tinf");
    }

    print!("interp ");
    let n = te_compile(expr, &[lk]).unwrap();
    let start = Instant::now();
    d = 0.0;
    for _j in 0..LOOPS {
        for i in 0..LOOPS {
            tmp = i as f64;
            d += te_eval_expr(&n);
        }
    }
    let eelapsed = start.elapsed().as_millis() as i32;
    // te_free is automatic via Drop

    /*Million floats per second input.*/
    print!(" {:.5}", d);
    if eelapsed != 0 {
        println!(
            "\t{:5}ms\t{:5}mfps",
            eelapsed,
            LOOPS as i64 * LOOPS as i64 / eelapsed as i64 / 1000
        );
    } else {
        println!("\tinf");
    }

    println!(
        "{:.2}% longer",
        (((eelapsed as f64 / nelapsed as f64) - 1.0) * 100.0)
    );
    println!();
}

fn a5(a: f64) -> f64 {
    a + 5.0
}

fn a55(a: f64) -> f64 {
    5.0 + a + 5.0
}

fn a5abs(a: f64) -> f64 {
    (a + 5.0).abs()
}

fn a52(a: f64) -> f64 {
    (a + 5.0) * 2.0
}

fn a10(a: f64) -> f64 {
    a + (5.0 * 2.0)
}

fn as_(a: f64) -> f64 {
    (a.powf(1.5) + a.powf(2.5)).sqrt()
}

fn al(a: f64) -> f64 {
    1.0 / (a + 1.0) + 2.0 / (a + 2.0) + 3.0 / (a + 3.0)
}

fn main() {
    bench("a+5", a5);
    bench("5+a+5", a55);
    bench("abs(a+5)", a5abs);
    bench("sqrt(a^1.5+a^2.5)", as_);
    bench("a+(5*2)", a10);
    bench("(a+5)*2", a52);
    bench("(1/(a+1)+2/(a+2)+3/(a+3))", al);
}