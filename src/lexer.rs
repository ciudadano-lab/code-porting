use crate::ast::TeVariable;
use crate::functions::{find_builtin, add, sub, mul, divide, pow, fmod};
use crate::token::*;
use std::os::raw::c_void;
use std::ptr;

pub struct State<'a> {
    pub start: &'a [u8],
    pub next: usize,
    pub error_pos: usize,
    pub type_: i32,
    pub value: f64,
    pub bound: *const f64,
    pub function: *const c_void,
    pub context: *mut c_void,
    pub lookup: &'a [TeVariable],
}

impl<'a> State<'a> {
    pub fn new(expression: &'a str, variables: &'a [TeVariable]) -> Self {
        State {
            start: expression.as_bytes(),
            next: 0,
            error_pos: 0,
            type_: TOK_NULL,
            value: 0.0,
            bound: ptr::null(),
            function: ptr::null(),
            context: ptr::null_mut(),
            lookup: variables,
        }
    }
}

fn find_lookup<'a>(s: &State<'a>, name: &[u8]) -> Option<&'a TeVariable> {
    for var in s.lookup {
        if var.name.as_bytes() == name {
            return Some(var);
        }
    }
    None
}

pub fn next_token(s: &mut State<'_>) {
    s.type_ = TOK_NULL;
    loop {
        if s.next >= s.start.len() {
            s.type_ = TOK_END;
            return;
        }
        let c = s.start[s.next];

        // Number
        if (c >= b'0' && c <= b'9') || c == b'.' {
            let start = s.next;
            // Consume a plausible numeric token. Signs are handled by the parser
            // as unary operators, so they should not be merged into a number.
            while s.next < s.start.len() {
                let ch = s.start[s.next];
                if (ch >= b'0' && ch <= b'9') || ch == b'.' || ch == b'e' || ch == b'E' {
                    s.next += 1;
                } else {
                    break;
                }
            }
            let slice = std::str::from_utf8(&s.start[start..s.next]).unwrap_or("");
            match slice.parse::<f64>() {
                Ok(v) => {
                    s.value = v;
                    s.type_ = TOK_NUMBER;
                }
                Err(_) => {
                    s.error_pos = start;
                    s.next = start + 1;
                    s.type_ = TOK_ERROR;
                }
            }
        }
        // Identifier / builtin / variable
        else if (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') {
            let start = s.next;
            s.next += 1;
            while s.next < s.start.len() {
                let ch = s.start[s.next];
                if (ch >= b'a' && ch <= b'z')
                    || (ch >= b'A' && ch <= b'Z')
                    || (ch >= b'0' && ch <= b'9')
                    || ch == b'_'
                {
                    s.next += 1;
                } else {
                    break;
                }
            }
            let name = &s.start[start..s.next];
            let var = find_lookup(s, name).or_else(|| find_builtin(name));
            match var {
                None => {
                    s.error_pos = start;
                    s.type_ = TOK_ERROR;
                }
                Some(var) => match type_mask(var.type_) {
                    TE_VARIABLE => {
                        s.type_ = TOK_VARIABLE;
                        s.bound = var.address as *const f64;
                    }
                    t if (TE_CLOSURE0..=TE_CLOSURE7).contains(&t) => {
                        s.context = var.context;
                        s.type_ = var.type_;
                        s.function = var.address;
                    }
                    t if (TE_FUNCTION0..=TE_FUNCTION7).contains(&t) => {
                        s.type_ = var.type_;
                        s.function = var.address;
                    }
                    _ => s.type_ = TOK_ERROR,
                },
            }
        }
        // Operator / punctuation
        else {
            s.next += 1;
            match c {
                b'+' => {
                    s.type_ = TOK_INFIX;
                    s.function = add as *const c_void;
                }
                b'-' => {
                    s.type_ = TOK_INFIX;
                    s.function = sub as *const c_void;
                }
                b'*' => {
                    s.type_ = TOK_INFIX;
                    s.function = mul as *const c_void;
                }
                b'/' => {
                    s.type_ = TOK_INFIX;
                    s.function = divide as *const c_void;
                }
                b'^' => {
                    s.type_ = TOK_INFIX;
                    s.function = pow as *const c_void;
                }
                b'%' => {
                    s.type_ = TOK_INFIX;
                    s.function = fmod as *const c_void;
                }
                b'(' => s.type_ = TOK_OPEN,
                b')' => s.type_ = TOK_CLOSE,
                b',' => s.type_ = TOK_SEP,
                b' ' | b'\t' | b'\n' | b'\r' => continue,
                _ => {
                    s.error_pos = s.next - 1;
                    s.type_ = TOK_ERROR;
                }
            }
        }

        if s.type_ != TOK_NULL {
            break;
        }
    }
}