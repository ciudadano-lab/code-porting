# Design Document

## Overview

This project is an idiomatic Rust migration of the original TinyExpr library while preserving its parsing algorithm, evaluation logic, and public behavior.

The primary design goal is **behavioral equivalence**, ensuring that valid expressions produce the same results as the original C implementation.

---

# Design Goals

The project was designed around the following principles:

- Preserve the original parsing algorithm
- Preserve operator precedence and associativity
- Preserve mathematical behavior
- Eliminate manual memory management
- Minimize complexity
- Follow idiomatic Rust practices
- Separate responsibilities into independent modules

---

# Architecture

```
                Expression
                     │
                     ▼
              ┌────────────┐
              │  lexer.rs  │
              └────────────┘
                     │
                     ▼
              ┌────────────┐
              │ token.rs   │
              └────────────┘
                     │
                     ▼
              ┌────────────┐
              │ parser.rs  │
              └────────────┘
                     │
                     ▼
              ┌────────────┐
              │  ast.rs    │
              └────────────┘
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
 ┌──────────────┐      ┌────────────────┐
 │ evaluator.rs │      │ optimizer.rs  │
 └──────────────┘      └────────────────┘
          │
          ▼
      Result (f64)

functions.rs supplies built-in functions.

error.rs provides parser and evaluation errors.

lib.rs exposes the public API.
```

---

# Module Responsibilities

## lib.rs

Provides the public API of the library.

Responsibilities:

- Export public functions
- Connect internal modules
- Hide implementation details

Equivalent to:

```
tinyexpr.h
```

---

## token.rs

Defines all lexical tokens.

Responsibilities:

- Token enumeration
- Token representation

No parsing logic is implemented here.

---

## lexer.rs

Responsible for lexical analysis.

Responsibilities:

- Read characters
- Skip whitespace
- Parse numbers
- Parse identifiers
- Produce tokens

Equivalent to:

```
next_token()
```

from the original C implementation.

---

## parser.rs

Implements the recursive-descent parser.

Responsibilities:

- Parse expressions
- Build the Abstract Syntax Tree
- Preserve the original grammar

Grammar:

```
list
 └── expr
      └── term
           └── factor
                └── power
                     └── base
```

The parser intentionally follows the same recursive structure as the original TinyExpr implementation.

---

## ast.rs

Defines the Abstract Syntax Tree.

Responsibilities:

- Expression node definitions
- Expression ownership
- Tree construction

No parsing or evaluation occurs here.

---

## evaluator.rs

Evaluates the AST.

Responsibilities:

- Recursive evaluation
- Variable lookup
- Function invocation

Equivalent to:

```
te_eval()
```

---

## optimizer.rs

Performs compile-time optimizations.

Responsibilities:

- Constant folding
- Simplify constant subexpressions

The optimization strategy matches the original TinyExpr implementation.

---

## functions.rs

Provides built-in mathematical functions.

Responsibilities:

- Register built-in functions
- Function lookup
- Mathematical helpers

Examples:

- sin
- cos
- tan
- sqrt
- log
- pow
- fac
- ncr
- npr

---

## error.rs

Defines error types used throughout the library.

Responsibilities:

- Parser errors
- Evaluation errors
- Error reporting

Rust's `Result` type replaces integer error codes used in C.

---

# Memory Management

## Original C

The original implementation allocates expression nodes using dynamic memory and requires explicit deallocation.

```
malloc()

free()
```

---

## Rust

The Rust implementation relies on ownership and automatic resource management.

Expression nodes are automatically released when they leave scope.

No manual deallocation is required.

---

# Error Handling

## Original C

Errors are reported through integer error positions and NULL pointers.

## Rust

Errors are represented using strongly typed enums and propagated using `Result`.

This improves readability and type safety without changing parser behavior.

---

# Behavioral Equivalence

The following properties are intentionally preserved:

- Grammar
- Operator precedence
- Operator associativity
- Expression evaluation order
- Built-in function behavior
- Variable lookup semantics

The parser algorithm itself has not been redesigned.

---

# Safety

The implementation is designed to minimize or eliminate the use of `unsafe`.

Rust's ownership model replaces manual memory management used in the original implementation, while the public API remains centered on `te_compile`, `te_eval_expr`, and `te_interp`.

---

# Testing Strategy

Correctness is verified through:

- Unit tests
- Smoke tests
- Differential testing against the original C implementation
- Benchmarks

---

# Future Improvements

Possible future work includes:

- Additional mathematical functions
- Configurable optimization levels
- Improved diagnostics
- no_std compatibility
- WebAssembly support

These improvements are intentionally outside the scope of the migration to preserve behavioral equivalence.