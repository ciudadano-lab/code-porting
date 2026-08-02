//! Built-in functions and operators.
//! Exact same set, order, and semantics as the original C table.

use crate::ast::TeVariable;
use crate::token::*;
use std::f64::{INFINITY, NAN};
use std::os::raw::c_void;
use std::ptr;

// ---------------------------------------------------------------------------
// Math wrappers (identical behaviour to C libm)
// ---------------------------------------------------------------------------
pub unsafe fn fabs(x: f64) -> f64 { x.abs() }
pub unsafe fn acos(x: f64) -> f64 { x.acos() }
pub unsafe fn asin(x: f64) -> f64 { x.asin() }
pub unsafe fn atan(x: f64) -> f64 { x.atan() }
pub unsafe fn atan2(y: f64, x: f64) -> f64 { y.atan2(x) }
pub unsafe fn ceil(x: f64) -> f64 { x.ceil() }
pub unsafe fn cos(x: f64) -> f64 { x.cos() }
pub unsafe fn cosh(x: f64) -> f64 { x.cosh() }
pub unsafe fn exp(x: f64) -> f64 { x.exp() }
pub unsafe fn floor(x: f64) -> f64 { x.floor() }
pub unsafe fn log(x: f64) -> f64 { x.ln() }
pub unsafe fn log10(x: f64) -> f64 { x.log10() }
pub unsafe fn pow(x: f64, y: f64) -> f64 { x.powf(y) }
pub unsafe fn sin(x: f64) -> f64 { x.sin() }
pub unsafe fn sinh(x: f64) -> f64 { x.sinh() }
pub unsafe fn sqrt(x: f64) -> f64 { x.sqrt() }
pub unsafe fn tan(x: f64) -> f64 { x.tan() }
pub unsafe fn tanh(x: f64) -> f64 { x.tanh() }
pub unsafe fn fmod(x: f64, y: f64) -> f64 { x % y }

// ---------------------------------------------------------------------------
// Special functions (exact C algorithms)
// ---------------------------------------------------------------------------
pub unsafe fn pi() -> f64 {
    3.14159265358979323846
}
pub unsafe fn e() -> f64 {
    2.71828182845904523536
}

pub unsafe fn fac(a: f64) -> f64 {
    if a < 0.0 {
        return NAN;
    }
    if a > u32::MAX as f64 {
        return INFINITY;
    }
    let ua = a as u32;
    let mut result: u64 = 1;
    for i in 1..=ua as u64 {
        if i > u64::MAX / result {
            return INFINITY;
        }
        result *= i;
    }
    result as f64
}

pub unsafe fn ncr(n: f64, r: f64) -> f64 {
    if n < 0.0 || r < 0.0 || n < r {
        return NAN;
    }
    if n > u32::MAX as f64 || r > u32::MAX as f64 {
        return INFINITY;
    }
    let un = n as u32 as u64;
    let mut ur = r as u32 as u64;
    let mut result: u64 = 1;
    if ur > un / 2 {
        ur = un - ur;
    }
    for i in 1..=ur {
        if result > u64::MAX / (un - ur + i) {
            return INFINITY;
        }
        result *= un - ur + i;
        result /= i;
    }
    result as f64
}

pub unsafe fn npr(n: f64, r: f64) -> f64 {
    ncr(n, r) * fac(r)
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------
pub unsafe fn add(a: f64, b: f64) -> f64 { a + b }
pub unsafe fn sub(a: f64, b: f64) -> f64 { a - b }
pub unsafe fn mul(a: f64, b: f64) -> f64 { a * b }
pub unsafe fn divide(a: f64, b: f64) -> f64 { a / b }
pub unsafe fn negate(a: f64) -> f64 { -a }
pub unsafe fn comma(_a: f64, b: f64) -> f64 { b }

// ---------------------------------------------------------------------------
// Builtin table (must stay alphabetical for binary search)
// ---------------------------------------------------------------------------
pub static FUNCTIONS: &[TeVariable] = &[
    TeVariable { name: "abs",   address: fabs as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "acos",  address: acos as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "asin",  address: asin as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "atan",  address: atan as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "atan2", address: atan2 as *const c_void, type_: TE_FUNCTION2 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "ceil",  address: ceil as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "cos",   address: cos as *const c_void,   type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "cosh",  address: cosh as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "e",     address: e as *const c_void,     type_: TE_FUNCTION0 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "exp",   address: exp as *const c_void,   type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "fac",   address: fac as *const c_void,   type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "floor", address: floor as *const c_void, type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "ln",    address: log as *const c_void,   type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    // TE_NAT_LOG would make "log" point to ln; default is log10
    TeVariable { name: "log",   address: log10 as *const c_void, type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "log10", address: log10 as *const c_void, type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "ncr",   address: ncr as *const c_void,   type_: TE_FUNCTION2 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "npr",   address: npr as *const c_void,   type_: TE_FUNCTION2 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "pi",    address: pi as *const c_void,    type_: TE_FUNCTION0 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "pow",   address: pow as *const c_void,   type_: TE_FUNCTION2 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "sin",   address: sin as *const c_void,   type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "sinh",  address: sinh as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "sqrt",  address: sqrt as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "tan",   address: tan as *const c_void,   type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
    TeVariable { name: "tanh",  address: tanh as *const c_void,  type_: TE_FUNCTION1 | TE_FLAG_PURE, context: ptr::null_mut() },
];

pub fn find_builtin(name: &[u8]) -> Option<&'static TeVariable> {
    let mut imin = 0usize;
    let mut imax = FUNCTIONS.len().saturating_sub(1);
    while imax >= imin {
        let i = imin + (imax - imin) / 2;
        let f = FUNCTIONS[i].name.as_bytes();
        let len = name.len().min(f.len());
        let cmp = name[..len].cmp(&f[..len]);
        let c = if cmp == std::cmp::Ordering::Equal {
            if name.len() == f.len() { 0 }
            else if name.len() < f.len() { -1 }
            else { 1 }
        } else if cmp == std::cmp::Ordering::Greater { 1 } else { -1 };

        if c == 0 {
            return Some(&FUNCTIONS[i]);
        } else if c > 0 {
            imin = i + 1;
        } else {
            if i == 0 { break; }
            imax = i - 1;
        }
    }
    None
}