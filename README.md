# tinyexpr_rs

tinyexpr_rs is a small Rust implementation of a TinyExpr-style expression parser and evaluator.

## Features

- arithmetic expressions with `+`, `-`, `*`, `/`, `%`, and `^`
- unary operators and parenthesized expressions
- built-in functions such as `sin`, `sqrt`, `log`, `fac`, `ncr`, and `npr`
- constants such as `pi` and `e`
- bound variables via `te_compile`

## Build and test

```bash
cargo test
cargo run
```

This project is intended to be a compact, self-contained parser/evaluator for experimentation and reuse in other Rust projects.
