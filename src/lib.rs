//! TinyExpr – tiny recursive-descent math expression parser / evaluator.
//! Faithful Rust port of the original C library by Lewis Van Winkle.
//!
//! Public API mirrors the C API as closely as possible.

mod token;
mod ast;
mod functions;
mod lexer;
mod parser;
mod optimizer;
mod evaluator;
mod error;

pub use ast::{Expr, TeVariable};
pub use error::{TeError, TeResult};
pub use token::*;

use evaluator::te_eval;
use optimizer::optimize;
use parser::te_compile as parse_compile;

/// Parse, optimise and evaluate in one step (mirrors `te_interp`).
pub fn te_interp(expression: &str) -> Result<f64, i32> {
    match te_compile(expression, &[]) {
        Ok(n) => Ok(unsafe { te_eval(&n) }),
        Err(e) => Err(e),
    }
}

/// Compile an expression with optional variable bindings (mirrors `te_compile`).
pub fn te_compile(
    expression: &str,
    variables: &[TeVariable],
) -> Result<Box<Expr>, i32> {
    let mut root = parse_compile(expression, variables)?;
    optimize(&mut root);
    Ok(root)
}

/// Evaluate a previously compiled expression (mirrors `te_eval`).
pub fn te_eval_expr(n: &Expr) -> f64 {
    unsafe { te_eval(n) }
}

/// Debug-print the expression tree (mirrors `te_print`).
pub fn te_print(n: &Expr) {
    te_print_depth(n, 0);
}

fn te_print_depth(n: &Expr, depth: usize) {
    print!("{:width$}", "", width = depth);
    match type_mask(n.type_) {
        TE_CONSTANT => println!("{}", n.value),
        TE_VARIABLE => println!("bound {:p}", n.bound),
        t if (TE_FUNCTION0..=TE_CLOSURE7).contains(&t) => {
            let ar = arity(n.type_) as usize;
            print!("f{}", ar);
            for p in &n.parameters {
                print!(" {:p}", p);
            }
            println!();
            for p in &n.parameters {
                te_print_depth(p, depth + 1);
            }
        }
        _ => println!("?"),
    }
}

// Re-export the most commonly needed items so users can write
// `use tinyexpr::*;`
pub use functions::FUNCTIONS;