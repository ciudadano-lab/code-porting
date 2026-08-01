# TinyExpr Rewrite Guide

## Purpose

This document captures the full architecture, public API, parser behavior, and implementation details of the TinyExpr project so it can be rewritten in another language with minimal guesswork.

## Project Summary

TinyExpr is a small, dependency-free math expression parser and evaluator written in C. It supports:

- arithmetic operators and precedence
- unary signs
- exponentiation
- built-in math functions such as sin, sqrt, ln, log10, abs, pow, fac, ncr, npr
- runtime-bound variables
- custom functions with arity
- syntax-tree compilation and later evaluation
- parse-error reporting with an error position

The implementation is centered around a recursive-descent parser that compiles an expression string into an abstract syntax tree (AST), then evaluates that tree.

---

## Repository Inventory

| File | Role |
| --- | --- |
| [tinyexpr.h](tinyexpr.h) | Public header with the API surface and public types |
| [tinyexpr.c](tinyexpr.c) | Core parser, tokenizer, AST construction, evaluator, optimizer |
| [README.md](README.md) | Project description, grammar, and usage documentation |
| [example.c](example.c) | Minimal example using te_interp |
| [example2.c](example2.c) | Example using compiled expressions with bound variables |
| [example3.c](example3.c) | Example using custom callables/functions |
| [smoke.c](smoke.c) | Large regression-style test suite |
| [repl.c](repl.c) | Interactive command-line expression evaluator |
| [benchmark.c](benchmark.c) | Performance comparison harness |
| [minctest.h](minctest.h) | Minimal unit-test helper header |
| [Makefile](Makefile) | Build targets for examples, tests, and benchmarks |
| [doc/e1.dot](doc/e1.dot) and [doc/e2.dot](doc/e2.dot) | Graphviz examples showing parse tree structure |

---

## Main Architectural Components

### 1. Public API

The public interface is defined in [tinyexpr.h](tinyexpr.h):

- te_interp(expression, error): parse, evaluate, and free in one step
- te_compile(expression, variables, var_count, error): build an AST with bound variable metadata
- te_eval(expr): evaluate a compiled expression tree
- te_free(expr): free the AST
- te_print(expr): debug-print the syntax tree

### 2. Core Data Structures

#### te_expr

Represents a node in the expression AST.

Key fields:

- type: identifies node kind
- value: used for constants
- bound: pointer to a bound variable value
- function: pointer to a callable implementation
- parameters: child nodes or context storage

#### te_variable

Represents a named symbol that can be resolved during parsing or evaluation.

Fields:

- name
- address
- type
- context

#### state

Parser state used by the tokenizer and recursive-descent parser.

Fields:

- start: pointer to beginning of the expression
- next: current scan position
- type: current token classification
- value: numeric literal value if token is a number
- bound: variable pointer if token is a variable
- function: function pointer for infix operators or built-ins
- context: closure context
- lookup: list of user-provided variables
- lookup_len: number of user variables

---

## Grammar Supported

The parser implements the following grammar:

```text
<list>      = <expr> {"," <expr>}
<expr>      = <term> {("+" | "-") <term>}
<term>      = <factor> {("*" | "/" | "%") <factor>}
<factor>    = <power> {"^" <power>}
<power>      = {("-" | "+")} <base>
<base>      = <constant>
            | <variable>
            | <function-0> {"(" ")"}
            | <function-1> <power>
            | <function-X> "(" <expr> {"," <expr>} ")"
            | "(" <list> ")"
```

Notes:

- Whitespace is ignored.
- Variable names start with a letter and may continue with letters, digits, or underscores.
- Numeric constants may be integers or floating-point values, including hexadecimal literals.
- The parser supports comma expressions, where the last expression value is returned.

---

## Parser and Evaluation Flow

### Tokenization

The tokenizer scans the input string one character at a time and converts it into token types such as:

- number
- variable
- operator/infix function
- opening/closing parenthesis
- separator comma
- end of input
- error

### Parsing Strategy

The parser uses recursive descent:

1. parse list
2. parse expression
3. parse term
4. parse factor/power
5. parse base

Each parser function builds AST nodes for the current grammar rule.

### Optimization

After parsing, the code calls optimize() to fold constant subexpressions when the node is marked pure. This reduces evaluation overhead for expressions like `2 + 3 * 4`.

### Evaluation

Evaluation is recursive and walks the AST. It resolves:

- constants directly
- variables by dereferencing their bound pointers
- functions by calling the stored callable
- closures by forwarding context as the first argument

---

## Built-in Functions and Constants

The implementation contains a built-in lookup table with functions and constants:

### Constants

- pi
- e

### Functions

- abs
- acos
- asin
- atan
- atan2
- ceil
- cos
- cosh
- exp
- fac
- floor
- ln
- log
- log10
- ncr
- npr
- pow
- sin
- sinh
- sqrt
- tan
- tanh

Built-ins are stored with an arity and a purity flag. Functions marked pure can be optimized if all arguments are constants.

---

## Important Semantics to Preserve

If the rewrite is to behave like the original, these behaviors must be preserved exactly or very closely:

### 1. Error Handling

- Parse failures return NaN for te_interp.
- The error position is returned as an integer index.
- A successful compilation returns 0 error.
- A syntax error should set the error index to the point where parsing failed.

### 2. Operator Precedence

The precedence is:

1. unary signs
2. exponentiation
3. multiplication/division/modulo
4. addition/subtraction
5. comma expressions

### 3. Exponentiation Associativity

The implementation supports a compile-time option for right-associative exponentiation. The default behavior is left-associative.

### 4. Unary Sign Handling

Expressions such as:

- `-5`
- `--5`
- `-x`
- `2^-3`

must be parsed correctly.

### 5. Function Arity

The parser distinguishes zero-argument, one-argument, and multi-argument functions. The type tags encode this as part of the node type.

### 6. Closures and Context

The system supports closures with a bound context pointer. This is how custom callables can receive extra state.

### 7. Memory Ownership

Every AST node owns its child nodes. `te_free()` must recursively free child nodes.

---

## Suggested Rewrite Structure

A straightforward rewrite should follow this layout:

```text
src/
  api/
    expressions.{ext}
    symbols.{ext}
  parser/
    tokenizer.{ext}
    parser.{ext}
    grammar.{ext}
  ast/
    node.{ext}
    builder.{ext}
  evaluator/
    evaluator.{ext}
    optimizer.{ext}
  builtins/
    math_builtins.{ext}
  tests/
    smoke_cases.{ext}
    parser_errors.{ext}
```

---

## Language-agnostic Mapping Plan

| TinyExpr concept | Rewrite target concept |
| --- | --- |
| te_expr | AST node object/class/record |
| te_variable | symbol table entry / variable descriptor |
| state | parser context object |
| token types | enum values |
| built-in table | map/list of built-in symbols |
| recursive parser | parser class with methods for each grammar rule |
| te_eval | recursive evaluator method |
| te_free | recursive node destructor |
| te_compile | parse + build AST + bind symbols |

---

## Recommended Implementation Order

### Phase 1: Minimal Parser

Implement the smallest working version that supports:

- numbers
- variables
- `+ - * / ^`
- parentheses
- unary signs
- built-in functions like `sqrt`, `sin`, `cos`

### Phase 2: AST and Evaluation

Add the AST node structure and recursive evaluator.

### Phase 3: Variable and Function Binding

Add support for user-defined variables and functions.

### Phase 4: Error Handling and Optimization

Match parse-error behavior and add constant-folding.

### Phase 5: Compatibility Tests

Port the smoke and syntax tests from [smoke.c](smoke.c) to the new language.

---

## Public API Sketch

A target-language version should expose something similar to:

```text
evaluate(expression: string) -> number
compile(expression: string, variables: list, error: out) -> expression_node
eval(node) -> number
destroy(node)
print_tree(node)
```

If the target language uses objects or classes, this can be wrapped as:

```text
ExpressionEngine.compile(expression, variables)
ExpressionEngine.evaluate(expression)
ExpressionEngine.eval(compiled_expression)
ExpressionEngine.dispose(compiled_expression)
```

---

## Test Coverage to Port

The test suite in [smoke.c](smoke.c) covers:

- basic arithmetic
- precedence and associativity
- unary handling
- function calls
- constants like pi and e
- comma expressions
- NaN propagation
- Infinity handling
- syntax error positions
- invalid expressions

The most important tests to preserve first are:

1. `1 + 2 * 3`
2. `(1 + 2) * 3`
3. `2^3^2`
4. `sqrt(100)`
5. `sin(0)`
6. `1,2,3`
7. `0/0`
8. `1%0`
9. `1+`
10. `a+5` when `a` is undefined

---

## Build and Runtime Notes

### Build Targets

The current Makefile builds:

- smoke
- smoke_pr
- repl
- bench
- example
- example2
- example3

A rewritten version should preserve similar targets or equivalent commands in the target ecosystem.

### Dependency Notes

The original implementation depends only on the C standard library and math library. The rewrite should preserve that spirit: no heavyweight runtime dependencies unless required by the target language.

---

## Risks and Gotchas

### 1. C-Style Pointer Semantics

The C version uses pointers heavily for both values and node relationships. The rewrite must clearly model ownership and references in the target language.

### 2. Function Pointer Differences

The original stores function pointers in AST nodes. In another language, this may become a function object, delegate, method reference, or closure.

### 3. Enum-Based Type Tags

The type system is encoded with bitmasks. The rewrite should use explicit enum values or a tagged union/object model instead of raw bit-packed integers if the target language supports it.

### 4. Error Position Accuracy

The error index must align with the original parser behavior. If the new implementation uses different tokenization rules, this needs careful testing.

### 5. Constant Folding

The optimizer is simple but important. It should be preserved or replicated to keep evaluation performance comparable.

---

## Minimal Rewrite Checklist

- [ ] Create tokenizer
- [ ] Create parser for grammar
- [ ] Create AST node model
- [ ] Implement evaluator
- [ ] Implement built-in constants/functions
- [ ] Implement variable/function binding
- [ ] Implement parse error reporting
- [ ] Implement memory cleanup/disposal
- [ ] Port smoke tests
- [ ] Port example programs
- [ ] Verify with a comparable benchmark

---

## Bottom Line

The heart of TinyExpr is not the math library itself; it is the combination of:

- a small recursive-descent parser
- a compact AST representation
- a recursive evaluator
- a symbol-binding layer for variables and functions

A rewrite in another language should preserve those three core pillars first, then add compatibility features and test coverage.
