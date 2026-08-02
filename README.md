# TinyExpr (Rust)

A Rust port of **TinyExpr**, a tiny recursive-descent mathematical expression parser and evaluator originally written in C by **Lewis Van Winkle (codeplea)**.

The goal of this project is to preserve the original parser's behavior while providing an idiomatic Rust implementation with safe memory management, modular architecture, and modern Rust tooling.

---

## Features

- Recursive-descent parser
- Expression evaluation
- Operator precedence and associativity
- Variables and constants
- Built-in mathematical functions
- Constant folding optimization
- Comprehensive test suite
- Benchmarks using Criterion
- Interactive REPL example

---

## Project Structure

```
tinyexpr/
│
├── Cargo.toml
├── README.md
├── LICENSE
│
├── src/
│   ├── lib.rs
│   ├── token.rs
│   ├── lexer.rs
│   ├── parser.rs
│   ├── ast.rs
│   ├── evaluator.rs
│   ├── optimizer.rs
│   ├── functions.rs
│   └── error.rs
│
├── examples/
│   ├── example.rs
│   ├── example2.rs
│   ├── example3.rs
│   └── repl.rs
│
├── tests/
│   ├── smoke.rs
│   └── parser_tests.rs
│
└── benches/
    └── benchmark.rs
```

---

## Building

```bash
cargo build
```

---

## Running Examples

Interactive calculator:

```bash
cargo run --example repl
```

Run the REPL with an inline expression:

```bash
cargo run --example repl -- -e "2+3"
```

Other examples:

```bash
cargo run --example example
cargo run --example example2 "2+3"
cargo run --example example3
```

---

## Running Tests

Run the automated test suite:

```bash
cargo test
```

The current workspace has also been verified with:

```bash
cargo test --lib
cargo test --test parser_tests
cargo test --test smoke
```

---

## Running Benchmarks

```bash
cargo bench
```

---

## Migration Goals

This project aims to preserve the original TinyExpr behavior while adopting Rust's ownership model and modular design.

The migration keeps:

- Parsing grammar
- Operator precedence
- Evaluation logic
- Mathematical behavior
- Public API semantics

The implementation avoids changing the parser algorithm or expression evaluation rules.

---

## Original Project

Original TinyExpr project by Lewis Van Winkle:

https://github.com/codeplea/tinyexpr

---

## License

This project follows the same **zlib License** as the original TinyExpr project.