use crate::ast::{Expr, TeVariable};
use crate::functions::{add, sub, mul, divide, pow, fmod, negate, comma};
use crate::lexer::{next_token, State};
use crate::token::*;
use std::os::raw::c_void;

fn base(s: &mut State<'_>) -> Option<Box<Expr>> {
    let typ = type_mask(s.type_);
    match typ {
        TOK_NUMBER => {
            let ret = Expr::new_constant(s.value);
            next_token(s);
            Some(ret)
        }
        TOK_VARIABLE => {
            let ret = Expr::new_variable(s.bound);
            next_token(s);
            Some(ret)
        }
        t if t == TE_FUNCTION0 || t == TE_CLOSURE0 => {
            let is_clo = is_closure(s.type_);
            let f = s.function;
            let ctx = s.context;
            let typ = s.type_;
            next_token(s);
            if s.type_ == TOK_OPEN {
                next_token(s);
                if s.type_ != TOK_CLOSE {
                    s.error_pos = s.next;
                    s.type_ = TOK_ERROR;
                    return None;
                }
                next_token(s);
            }
            if is_clo {
                Some(Expr::new_closure(typ, f, ctx, vec![]))
            } else {
                Some(Expr::new_func(typ, f, vec![]))
            }
        }
        t if t == TE_FUNCTION1 || t == TE_CLOSURE1 => {
            let is_clo = is_closure(s.type_);
            let f = s.function;
            let ctx = s.context;
            let typ = s.type_;
            next_token(s);
            let p0 = power(s)?;
            if is_clo {
                Some(Expr::new_closure(typ, f, ctx, vec![p0]))
            } else {
                Some(Expr::new_func(typ, f, vec![p0]))
            }
        }
        t if (TE_FUNCTION2..=TE_FUNCTION7).contains(&t)
            || (TE_CLOSURE2..=TE_CLOSURE7).contains(&t) =>
        {
            let ar = arity(s.type_) as usize;
            let is_clo = is_closure(s.type_);
            let f = s.function;
            let ctx = s.context;
            let typ = s.type_;
            next_token(s);
            if s.type_ != TOK_OPEN {
                s.error_pos = s.next;
                s.type_ = TOK_ERROR;
                return None;
            }
            let mut params = Vec::with_capacity(ar);
            for i in 0..ar {
                next_token(s);
                let e = expr(s)?;
                params.push(e);
                if i + 1 < ar && s.type_ != TOK_SEP {
                    s.error_pos = s.next;
                    s.type_ = TOK_ERROR;
                    return None;
                }
            }
            if s.type_ != TOK_CLOSE {
                s.error_pos = s.next;
                s.type_ = TOK_ERROR;
                return None;
            }
            next_token(s);
            if is_clo {
                Some(Expr::new_closure(typ, f, ctx, params))
            } else {
                Some(Expr::new_func(typ, f, params))
            }
        }
        TOK_OPEN => {
            next_token(s);
            let ret = list(s)?;
            if s.type_ != TOK_CLOSE {
                s.error_pos = s.next;
                s.type_ = TOK_ERROR;
                return None;
            }
            next_token(s);
            Some(ret)
        }
        _ => {
            s.error_pos = s.next;
            s.type_ = TOK_ERROR;
            None
        }
    }
}

fn power(s: &mut State<'_>) -> Option<Box<Expr>> {
    let mut sign = 1i32;
    while s.type_ == TOK_INFIX
        && (s.function == add as *const c_void || s.function == sub as *const c_void)
    {
        if s.function == sub as *const c_void {
            sign = -sign;
        }
        next_token(s);
    }
    let b = base(s)?;
    if sign == 1 {
        Some(b)
    } else {
        Some(Expr::new_func(
            TE_FUNCTION1 | TE_FLAG_PURE,
            negate as *const c_void,
            vec![b],
        ))
    }
}

// Default left-associative power (TE_POW_FROM_RIGHT not defined)
fn factor(s: &mut State<'_>) -> Option<Box<Expr>> {
    let mut ret = power(s)?;
    while s.type_ == TOK_INFIX && s.function == pow as *const c_void {
        let t = s.function;
        next_token(s);
        let p = power(s)?;
        ret = Expr::new_func(TE_FUNCTION2 | TE_FLAG_PURE, t, vec![ret, p]);
    }
    Some(ret)
}

fn term(s: &mut State<'_>) -> Option<Box<Expr>> {
    let mut ret = factor(s)?;
    while s.type_ == TOK_INFIX
        && (s.function == mul as *const c_void
            || s.function == divide as *const c_void
            || s.function == fmod as *const c_void)
    {
        let t = s.function;
        next_token(s);
        let f = factor(s)?;
        ret = Expr::new_func(TE_FUNCTION2 | TE_FLAG_PURE, t, vec![ret, f]);
    }
    Some(ret)
}

fn expr(s: &mut State<'_>) -> Option<Box<Expr>> {
    let mut ret = term(s)?;
    while s.type_ == TOK_INFIX
        && (s.function == add as *const c_void || s.function == sub as *const c_void)
    {
        let t = s.function;
        next_token(s);
        let te = term(s)?;
        ret = Expr::new_func(TE_FUNCTION2 | TE_FLAG_PURE, t, vec![ret, te]);
    }
    Some(ret)
}

fn list(s: &mut State<'_>) -> Option<Box<Expr>> {
    let mut ret = expr(s)?;
    while s.type_ == TOK_SEP {
        next_token(s);
        let e = expr(s)?;
        ret = Expr::new_func(
            TE_FUNCTION2 | TE_FLAG_PURE,
            comma as *const c_void,
            vec![ret, e],
        );
    }
    Some(ret)
}

/// Compile an expression (mirrors `te_compile`).
pub fn te_compile(
    expression: &str,
    variables: &[TeVariable],
) -> Result<Box<Expr>, i32> {
    let mut s = State::new(expression, variables);
    next_token(&mut s);
    let root = match list(&mut s) {
        Some(r) => r,
        None => {
            let err = if s.next == 0 {
                1
            } else if s.next >= s.start.len() {
                s.next as i32
            } else if s.error_pos > 0 {
                (s.error_pos as i32) + 1
            } else {
                (s.next as i32) + 1
            };
            return Err(err);
        }
    };
    if s.type_ != TOK_END {
        let err = if s.next == 0 {
            1
        } else if s.next >= s.start.len() {
            s.next as i32
        } else if s.error_pos > 0 {
            (s.error_pos as i32) + 1
        } else {
            (s.next as i32) + 1
        };
        return Err(err);
    }
    Ok(root)
}