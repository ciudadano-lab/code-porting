use crate::token::*;
use std::os::raw::c_void;
use std::ptr;

/// Owned expression tree node.
/// Layout and semantics match the original `te_expr`.
#[derive(Debug)]
pub struct Expr {
    pub type_: i32,
    pub value: f64,
    pub bound: *const f64,
    pub function: *const c_void,
    pub context: *mut c_void,
    pub parameters: Vec<Box<Expr>>,
}

impl Expr {
    pub fn new_constant(v: f64) -> Box<Expr> {
        Box::new(Expr {
            type_: TE_CONSTANT,
            value: v,
            bound: ptr::null(),
            function: ptr::null(),
            context: ptr::null_mut(),
            parameters: Vec::new(),
        })
    }

    pub fn new_variable(bound: *const f64) -> Box<Expr> {
        Box::new(Expr {
            type_: TE_VARIABLE,
            value: 0.0,
            bound,
            function: ptr::null(),
            context: ptr::null_mut(),
            parameters: Vec::new(),
        })
    }

    pub fn new_func(t: i32, f: *const c_void, params: Vec<Box<Expr>>) -> Box<Expr> {
        Box::new(Expr {
            type_: t,
            value: 0.0,
            bound: ptr::null(),
            function: f,
            context: ptr::null_mut(),
            parameters: params,
        })
    }

    pub fn new_closure(
        t: i32,
        f: *const c_void,
        ctx: *mut c_void,
        params: Vec<Box<Expr>>,
    ) -> Box<Expr> {
        Box::new(Expr {
            type_: t,
            value: 0.0,
            bound: ptr::null(),
            function: f,
            context: ctx,
            parameters: params,
        })
    }
}

/// Public variable binding (mirrors `te_variable`).
#[derive(Clone, Copy)]
pub struct TeVariable {
    pub name: &'static str,
    pub address: *const c_void,
    pub type_: i32,
    pub context: *mut c_void,
}

// The built-in function table is immutable and only exposes read-only function
// pointers, so it is safe to share across threads.
unsafe impl Send for TeVariable {}
unsafe impl Sync for TeVariable {}