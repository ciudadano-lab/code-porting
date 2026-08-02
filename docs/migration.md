# Migration Guide

## Overview

This project is a Rust migration of the original **TinyExpr** library by Lewis Van Winkle.

The objective of the migration is to preserve the functionality, parsing algorithm, and evaluation behavior of the original C implementation while adopting Rust's ownership model and modular architecture.

The migration intentionally avoids changing the parser grammar or expression evaluation semantics.

---

# Migration Objectives

The primary objectives are:

- Preserve parser behavior
- Preserve evaluation logic
- Preserve operator precedence
- Preserve mathematical results
- Replace manual memory management with Rust ownership
- Improve maintainability through modular design
- Follow idiomatic Rust practices

---

# Source Mapping

The original TinyExpr implementation consists primarily of two files:

```
tinyexpr.c
tinyexpr.h
```

These have been reorganized into the following Rust modules.

| Original C | Rust Module | Purpose |
|------------|-------------|---------|
| tinyexpr.h | lib.rs | Public library interface |
| Token definitions | token.rs | Token representation |
| next_token() | lexer.rs | Lexical analysis |
| base(), power(), factor(), term(), expr(), list() | parser.rs | Recursive-descent parser |
| struct te_expr | ast.rs | Abstract Syntax Tree |
| te_eval() | evaluator.rs | Expression evaluation |
| optimize() | optimizer.rs | Constant folding |
| Built-in functions | functions.rs | Mathematical functions |
| Error handling | error.rs | Strongly typed errors |

---

# Memory Management

## Original C

The original implementation dynamically allocates expression nodes using:

```
malloc()
free()
```

The caller is responsible for freeing compiled expressions.

---

## Rust

The Rust implementation relies on ownership and automatic resource management.

Expression trees are automatically released when they go out of scope.

No explicit deallocation is required.

This removes the possibility of:

- Memory leaks
- Double frees
- Dangling pointers

without changing program behavior.

---

# Error Handling

## Original C

Errors are reported using:

- NULL pointers
- Integer error positions

## Rust

Errors are represented using:

```
Result<T, TeError>
```

This provides stronger type safety while preserving equivalent failure conditions.

---

# Parser Preservation

The recursive-descent parser has been preserved.

Grammar:

```
list
 └── expr
      └── term
           └── factor
                └── power
                     └── base
```

Operator precedence and associativity remain unchanged.

---

# Optimization

The original TinyExpr performs compile-time constant folding.

This behavior is preserved in:

```
optimizer.rs
```

Expressions consisting entirely of constants are simplified before evaluation.

---

# Public API

The Rust implementation exposes an API equivalent in purpose to the original C library.

Original C:

```
te_compile()

te_eval()

te_interp()

te_free()
```

Rust:

```
te_compile()
te_eval_expr()
te_interp()
```

Memory deallocation is handled automatically by Rust and therefore does not require a direct equivalent of `te_free()`.

---

# Testing Strategy

Correctness is verified using:

- Parser regression tests
- Smoke tests
- Benchmarks

The objective is to verify behavioral equivalence rather than merely successful compilation.

---

# Performance

Performance is evaluated separately using Criterion benchmarks.

Benchmark methodology is documented in:

```
docs/benchmark.md
```

---

# Behavioral Equivalence

The following aspects were intentionally preserved:

- Grammar
- Parsing order
- Operator precedence
- Operator associativity
- Evaluation order
- Built-in mathematical functions
- Variable handling
- Optimization behavior

No intentional semantic changes have been introduced.

---

# Rust Improvements

The migration introduces several engineering improvements without changing behavior.

- Automatic memory management
- Modular architecture
- Strongly typed errors
- Safer ownership model
- Standard Cargo project structure
- Integrated testing
- Benchmark support

These improvements affect maintainability and safety rather than parser semantics.

---

# Known Differences

The implementation is intended to preserve observable behavior.

Any differences from the original implementation should be documented here if discovered during testing.

At the time of writing, no intentional behavioral differences have been introduced.

---

# Conclusion

This migration preserves the original TinyExpr parser and evaluator while adopting Rust's language features to improve safety, maintainability, and project organization.

The focus of the migration is correctness and behavioral equivalence rather than redesigning the original implementation.