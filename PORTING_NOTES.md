# Porting notes: tinyexpr.c -> Rust

This document maps every function in the original `tinyexpr.c`/`tinyexpr.h`
to where it landed in this crate, and lists every place the port's observable
behavior differs from the C original (there are only a few, and each is
explained).

## File/function map

| C (`tinyexpr.c` / `.h`)                | Rust                                                | Notes |
|-----------------------------------------|------------------------------------------------------|-------|
| `te_expr` struct + `type` bitfield       | `ast::Expr`, `ast::ExprKind`, `ast::Binding`          | Bit-packed `type`/`address` union replaced by an enum with one correctly-typed variant per arity/shape. Behaviorally identical dispatch, safer representation. |
| `te_variable` struct                     | `ast::Variable`                                       | Same 4 logical fields (name, address/binding, type/purity, context). |
| `TE_VARIABLE`/`TE_FUNCTIONn`/`TE_CLOSUREn`/`TE_FLAG_PURE` | `Binding` enum variants + `Variable::is_pure` | Same shapes (arity 0-7, plain or closure), same purity flag, no bit math needed. |
| `TYPE_MASK`, `ARITY`, `IS_PURE`, `IS_FUNCTION`, `IS_CLOSURE` macros | `Binding::arity()`, `Binding::is_variable()`, `is_pure` field | Same logical predicates, computed by `match` instead of bitmasking. |
| `new_expr`, `NEW_EXPR`, `CHECK_NULL`      | `Expr::constant`, `Expr::variable`, `Expr::call`      | No allocation-failure path (see divergence #1 below); manual size/memset math replaced by `Vec`/`Box`. |
| `te_free_parameters`, `te_free`           | (nothing) / `te_free()` no-op                         | `Vec<Expr>`'s `Drop` already recursively frees the tree; see divergence #2. |
| `pi`, `e`, `fac`, `ncr`, `npr`            | `builtins::pi`, `builtins::e`, `builtins::fac`, `builtins::ncr`, `builtins::npr` | Line-for-line, including the exact overflow-to-`INFINITY` behavior. |
| `functions[]` static table + `find_builtin` | `builtins::BUILTINS` + `builtins::find_builtin`     | Same 24 entries, same alphabetical order (binary search relies on it, like the original), same default `log`=`log10` choice. |
| `find_lookup`                             | inlined `.iter().find(...)` in `parser::Lexer::next_token` | Same linear scan over caller-supplied variables, same "custom lookup wins over builtin" priority. |
| `add`, `sub`, `mul`, `divide`, `negate`, `comma` | `parser::add_/sub_/mul_/div_/negate_/comma_`      | Line-for-line. |
| `next_token`                              | `parser::Lexer::next_token` (+ `scan_number`)         | Same token grammar; see divergence #3 for the one hardening change (lone `.`). |
| `base`                                    | `parser::Parser::parse_base`                          | Same grammar, same per-arity argument-parsing loop and off-by-one-safe arity check (`i != arity-1`). |
| `power`                                    | `parser::Parser::parse_power`                         | Same sign-accumulation loop. |
| `factor` (default, non-`TE_POW_FROM_RIGHT`) | `parser::Parser::parse_factor`                       | The C source ships with `TE_POW_FROM_RIGHT` undefined, so this port only implements that (actually-compiled) branch; see divergence #4. |
| `term`                                     | `parser::Parser::parse_term`                          | Same. |
| `expr`                                     | `parser::Parser::parse_expr`                          | Same. |
| `list`                                     | `parser::Parser::parse_list`                          | Same. |
| `te_eval` + `TE_FUN`/`M` macros            | `Expr::eval` + `ast::call_binding`                    | Same per-arity dispatch, same left-to-right argument evaluation order. |
| `optimize`                                 | `Expr::optimize`                                      | Same recursion rule, *including* the original's quirk that an impure call's arguments are never visited/folded. |
| `te_compile`                               | `te_compile` (in `lib.rs`)                            | Same overall shape; see divergence #1 (no `-1`/OOM case) and #5 (error position bookkeeping). |
| `te_interp`                                | `te_interp`                                           | Same. |
| `pn`, `te_print`                           | `Expr::print_at`, `Expr::print`                        | Same debug-dump shape (arity, child pointers, recursion). |

## Intentional divergences (and why each one is safe)

1. **No "compile totally failed" (`error == -1`) case.**
   In C, `te_compile` can fail before parsing even starts if `new_expr`'s
   `malloc` returns `NULL`. Rust's global allocator aborts the process on
   allocation failure instead of returning a null pointer that call sites
   would have to check, so there is nothing for this port to represent here
   -- it's not a difference in parsing *logic*, it's a difference in what
   "out of memory" does at the language level, and it's out of scope for a
   correct translation of the expression grammar/evaluator.

2. **No explicit `te_free`.**
   `te_free`/`te_free_parameters` manually recursed the tree and called
   `free()` on every node. `Expr`'s fields are `Vec<Expr>`/`Box`-backed, so
   Rust's ordinary `Drop` glue does the exact same recursive teardown
   automatically when an `Expr` goes out of scope. A `te_free(expr)` no-op
   function is still exported for call-site parity.

3. **A lone `.` with no digits is a lex error instead of an infinite loop.**
   In the C lexer, seeing `.` or a digit unconditionally routes into
   `strtod`. For genuinely malformed input like a standalone `.` with no
   digits anywhere near it, `strtod` reports "no conversion" and leaves its
   `endptr` unchanged, which -- because nothing else advances the read
   position either -- can make the original hang parsing the same
   character forever instead of terminating with an error. This port scans
   for at least one digit before accepting a numeric token, and treats
   the no-digit case as a lex error (which reports a syntax error at that
   position, same as any other unrecognized token, and always terminates).
   This does not change the result for any expression that contains at
   least one digit around a decimal point -- i.e. every valid numeric
   literal the original correctly parses is parsed identically here.

4. **Only the shipped (default) `factor()` variant is implemented.**
   `tinyexpr.c` has two implementations of `factor()` gated by
   `#ifdef TE_POW_FROM_RIGHT`, but that macro is commented out in the
   shipped source, so only the `#else` branch (left-associative `^`,
   i.e. `a^b^c == (a^b)^c`) is ever actually compiled. This port implements
   that branch -- the one that actually ships -- and doesn't carry the
   dead alternate branch forward. (`tests/smoke.rs::test_pow` uses exactly
   the C test file's `#else` case list to confirm this.)

5. **Error-position bookkeeping is restructured, not just relocated.**
   The C version computes `error = s->next - s->start` once, at the very
   end of `te_compile`, relying on the invariant that `next_token()` is
   never called again once an error condition has been latched (every
   error site either doesn't call `next_token` again, or -- in the lexer
   itself -- advances exactly as far as a real token would). This port
   makes that invariant explicit and structural: a `Parser::fail()` helper
   marks the parse as failed *and* pins the current token to a sentinel
   `Error` value, so no enclosing grammar-loop condition can accidentally
   see a stale-but-still-token-shaped value and advance further by mistake.
   The resulting reported position is identical to the original's for
   every case in `smoke.c::test_syntax` (verified in `tests/smoke.rs`);
   this divergence is about *how* the invariant is enforced, not a change
   in the reported positions themselves.

None of the above change the result of evaluating any expression the
original evaluates without hitting one of these exact edge cases, and
`tests/smoke.rs` (a full translation of `smoke.c`) plus the four example
programs are used to check that directly.
