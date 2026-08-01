//! A safe Rust port of [TinyExpr](https://github.com/codeplea/tinyexpr), a
//! tiny recursive-descent math expression parser/evaluator originally
//! written in C by Lewis Van Winkle.
//!
//! This crate reimplements the exact same grammar, operator precedence and
//! associativity, built-in function set, constant-folding pass, and error
//! reporting convention (a byte offset into the input) as the original
//! `tinyexpr.c` / `tinyexpr.h`. See `PORTING_NOTES.md` in the repository
//! root for a full function-by-function map from the C source to this
//! crate, and a list of the small number of deliberate, documented
//! divergences (all of them either (a) situations that only differ because
//! Rust's memory model can't fail the way C's `malloc` can, or (b) safety
//! improvements against undefined/hanging behavior on malformed input that
//! do not change the result for any input the original handles correctly).
//!
//! # Example
//! ```
//! let value = tinyexpr::te_interp("(1 + 2) * sqrt(9)").unwrap();
//! assert_eq!(value, 9.0);
//! ```
//!
//! # Binding variables
//! ```
//! let mut x = 3.0f64;
//! let vars = [tinyexpr::Variable::variable("x", &x as *const f64)];
//! let expr = tinyexpr::te_compile("x*x + 1", &vars).unwrap();
//! assert_eq!(expr.eval(), 10.0);
//! x = 4.0;
//! assert_eq!(expr.eval(), 17.0); // re-evaluating re-reads the bound address
//! ```

mod ast;
mod builtins;
mod parser;

pub use ast::{Binding, Expr, Variable};

/// Port of `te_compile()`.
///
/// Parses `expression`, binding any identifiers found in `variables` (and
/// falling back to the built-in constants/functions), and returns the
/// compiled, constant-folded expression tree.
///
/// On success, returns `Ok(expr)`. On a syntax error, returns
/// `Err(position)` where `position` is a 1-based-ish byte offset into
/// `expression` at which parsing stopped -- exactly the value the original
/// writes through its `int *error` out-parameter (a nonzero value; `0`
/// meant success in the original's convention, which is why this port
/// isn't representable as `Err(0)`).
///
/// Note: unlike the C original, this cannot fail due to memory allocation
/// (Rust's allocator aborts the process on out-of-memory rather than
/// returning null), so there is no analogue of the original's `error = -1`
/// "compile totally failed" case.
pub fn te_compile<'a>(expression: &str, variables: &[Variable]) -> Result<Expr, usize> {
    let mut p = parser::Parser::new(expression, variables);
    let mut root = p.parse_list();
    if !p.ok() {
        return Err(p.error_position());
    }
    root.optimize();
    Ok(root)
}

/// Port of `te_interp()`: parses `expression` with no variables bound,
/// evaluates it once, and returns the result (or the error position on a
/// syntax error). The original returns `NAN` on error via its `double`
/// return type plus an out-parameter; this port surfaces the same
/// information as a `Result` instead.
pub fn te_interp(expression: &str) -> Result<f64, usize> {
    te_compile(expression, &[]).map(|e| e.eval())
}

/// Port of `te_free()`. Dropping an `Expr` already recursively frees the
/// whole tree (see the doc comment on `Expr`), so this is a no-op that
/// simply takes ownership -- provided only so call sites that mirror the C
/// API 1:1 have something to call. `te_free(None)` in the original was
/// documented as safe; here that's just... not calling this function at
/// all, since `Option::None` never owns an `Expr` to begin with.
pub fn te_free(_expr: Expr) {}
