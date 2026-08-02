use crate::ast::Expr;
use crate::token::*;
use std::f64::NAN;
use std::os::raw::c_void;

type TeFun0 = unsafe fn() -> f64;
type TeFun1 = unsafe fn(f64) -> f64;
type TeFun2 = unsafe fn(f64, f64) -> f64;
type TeFun3 = unsafe fn(f64, f64, f64) -> f64;
type TeFun4 = unsafe fn(f64, f64, f64, f64) -> f64;
type TeFun5 = unsafe fn(f64, f64, f64, f64, f64) -> f64;
type TeFun6 = unsafe fn(f64, f64, f64, f64, f64, f64) -> f64;
type TeFun7 = unsafe fn(f64, f64, f64, f64, f64, f64, f64) -> f64;

type TeClo0 = unsafe fn(*mut c_void) -> f64;
type TeClo1 = unsafe fn(*mut c_void, f64) -> f64;
type TeClo2 = unsafe fn(*mut c_void, f64, f64) -> f64;
type TeClo3 = unsafe fn(*mut c_void, f64, f64, f64) -> f64;
type TeClo4 = unsafe fn(*mut c_void, f64, f64, f64, f64) -> f64;
type TeClo5 = unsafe fn(*mut c_void, f64, f64, f64, f64, f64) -> f64;
type TeClo6 = unsafe fn(*mut c_void, f64, f64, f64, f64, f64, f64) -> f64;
type TeClo7 = unsafe fn(*mut c_void, f64, f64, f64, f64, f64, f64, f64) -> f64;

/// Evaluate an expression tree (mirrors `te_eval`).
pub unsafe fn te_eval(n: &Expr) -> f64 {
    match type_mask(n.type_) {
        TE_CONSTANT => n.value,
        TE_VARIABLE => *n.bound,
        t if (TE_FUNCTION0..=TE_FUNCTION7).contains(&t) => {
            let ar = arity(n.type_) as usize;
            match ar {
                0 => {
                    let f: TeFun0 = std::mem::transmute(n.function);
                    f()
                }
                1 => {
                    let f: TeFun1 = std::mem::transmute(n.function);
                    f(te_eval(&n.parameters[0]))
                }
                2 => {
                    let f: TeFun2 = std::mem::transmute(n.function);
                    f(te_eval(&n.parameters[0]), te_eval(&n.parameters[1]))
                }
                3 => {
                    let f: TeFun3 = std::mem::transmute(n.function);
                    f(
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                    )
                }
                4 => {
                    let f: TeFun4 = std::mem::transmute(n.function);
                    f(
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                    )
                }
                5 => {
                    let f: TeFun5 = std::mem::transmute(n.function);
                    f(
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                        te_eval(&n.parameters[4]),
                    )
                }
                6 => {
                    let f: TeFun6 = std::mem::transmute(n.function);
                    f(
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                        te_eval(&n.parameters[4]),
                        te_eval(&n.parameters[5]),
                    )
                }
                7 => {
                    let f: TeFun7 = std::mem::transmute(n.function);
                    f(
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                        te_eval(&n.parameters[4]),
                        te_eval(&n.parameters[5]),
                        te_eval(&n.parameters[6]),
                    )
                }
                _ => NAN,
            }
        }
        t if (TE_CLOSURE0..=TE_CLOSURE7).contains(&t) => {
            let ar = arity(n.type_) as usize;
            match ar {
                0 => {
                    let f: TeClo0 = std::mem::transmute(n.function);
                    f(n.context)
                }
                1 => {
                    let f: TeClo1 = std::mem::transmute(n.function);
                    f(n.context, te_eval(&n.parameters[0]))
                }
                2 => {
                    let f: TeClo2 = std::mem::transmute(n.function);
                    f(
                        n.context,
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                    )
                }
                3 => {
                    let f: TeClo3 = std::mem::transmute(n.function);
                    f(
                        n.context,
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                    )
                }
                4 => {
                    let f: TeClo4 = std::mem::transmute(n.function);
                    f(
                        n.context,
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                    )
                }
                5 => {
                    let f: TeClo5 = std::mem::transmute(n.function);
                    f(
                        n.context,
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                        te_eval(&n.parameters[4]),
                    )
                }
                6 => {
                    let f: TeClo6 = std::mem::transmute(n.function);
                    f(
                        n.context,
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                        te_eval(&n.parameters[4]),
                        te_eval(&n.parameters[5]),
                    )
                }
                7 => {
                    let f: TeClo7 = std::mem::transmute(n.function);
                    f(
                        n.context,
                        te_eval(&n.parameters[0]),
                        te_eval(&n.parameters[1]),
                        te_eval(&n.parameters[2]),
                        te_eval(&n.parameters[3]),
                        te_eval(&n.parameters[4]),
                        te_eval(&n.parameters[5]),
                        te_eval(&n.parameters[6]),
                    )
                }
                _ => NAN,
            }
        }
        _ => NAN,
    }
}