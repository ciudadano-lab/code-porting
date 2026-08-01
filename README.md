# tinyexpr (Rust)

A safe Rust port of [TinyExpr](https://github.com/codeplea/tinyexpr), the
tiny recursive-descent math expression parser and evaluation engine
originally written in C by Lewis Van Winkle (codeplea).

This is a faithful, function-by-function translation, not a rewrite: the
grammar, operator precedence/associativity, built-in function set, constant
folding pass, and error-position convention are all unchanged from the C
original. See [`PORTING_NOTES.md`](./PORTING_NOTES.md) for a line-by-line map
from `tinyexpr.c` to this crate, and a short, complete list of the only
places behavior deliberately differs (all either forced by Rust's memory
model, or documented robustness fixes that don't change results for any
input the C version handled correctly).

## Features

- Pure Rust, no dependencies, no `unsafe` outside of the two places that are
  inherent to the original design (dereferencing a caller-bound variable
  address, and dispatching a closure's opaque `context` pointer).
- Same operator precedence and grammar as the original.
- Same 24 built-in functions/constants (`sin`, `sqrt`, `ln`, `log`, `pi`, ...).
- Custom variables and functions (arity 0-7), including "closures" that
  additionally take a caller-supplied context pointer, exactly like the
  original's `TE_CLOSUREn`.
- Late variable binding: compile once, mutate the bound value, `eval()`
  again -- no recompilation needed.
- Same constant-folding optimizer (`te_compile` returns an already-folded
  tree when every sub-expression is made of pure, constant inputs).
- Same error reporting: a syntax error returns the same byte offset into
  the input the C version would have written to its `int *error`
  out-parameter.

## Example

```rust
let value = tinyexpr::te_interp("(1 + 2) * sqrt(9)").unwrap();
assert_eq!(value, 9.0);
```

```rust
let mut x = 3.0f64;
let vars = [tinyexpr::Variable::variable("x", &x as *const f64)];
let expr = tinyexpr::te_compile("x*x + 1", &vars).unwrap();
assert_eq!(expr.eval(), 10.0);
x = 4.0;
assert_eq!(expr.eval(), 17.0); // re-evaluating re-reads the bound address
```

## API

| Original (C)                                          | This crate                                                |
|---------------------------------------------------------|------------------------------------------------------------|
| `double te_interp(const char *e, int *error)`            | `fn te_interp(e: &str) -> Result<f64, usize>`               |
| `te_expr *te_compile(const char *e, const te_variable *vars, int n, int *error)` | `fn te_compile(e: &str, vars: &[Variable]) -> Result<Expr, usize>` |
| `double te_eval(const te_expr *n)`                       | `Expr::eval(&self) -> f64`                                  |
| `void te_print(const te_expr *n)`                        | `Expr::print(&self)`                                        |
| `void te_free(te_expr *n)`                               | dropping an `Expr` (or the provided no-op `te_free(expr)`)  |

`error == 0` in C means "no error"; here that's `Ok(...)`. A nonzero `error`
(a byte position) becomes `Err(position)`, same value.

## Building the built-in functions/constants table

Same 24 entries, same names, same default compile-time choices as the
shipped C source (`TE_NAT_LOG` and `TE_POW_FROM_RIGHT` both left off, so
`log` means base-10 log and `a^b^c` means `(a^b)^c`, matching
`tinyexpr.c`'s actual defaults):

`abs acos asin atan atan2 ceil cos cosh e exp fac floor ln log log10 ncr npr
pi pow sin sinh sqrt tan tanh`

## Testing

`tests/smoke.rs` is a line-by-line translation of the original's
`smoke.c` test suite (`test_results`, `test_syntax`, `test_nans`,
`test_infs`, `test_variables`, `test_functions`, `test_dynamic`,
`test_closure`, `test_optimize`, `test_pow`, `test_combinatorics`) --
same expressions, same expected results, same expected error positions.

```sh
cargo test
```

## Examples

Direct ports of `example.c`, `example2.c`, `example3.c`, and `repl.c`:

```sh
cargo run --example example
cargo run --example example2 -- "x^2+y"
cargo run --example example3
cargo run --example repl
```

## License

Zlib, same as the original.
