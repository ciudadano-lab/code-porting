//! Built-in constants and functions, ported 1:1 from the `functions[]` table
//! and helper functions (`pi`, `e`, `fac`, `ncr`, `npr`) in the original
//! `tinyexpr.c`.
//!
//! The table below is kept in alphabetical order by name, exactly like the
//! original, because `find_builtin` relies on that order to binary search it.

use crate::ast::Binding;

/// `pi()` — same constant as the C source (not `std::f64::consts::PI`,
/// to preserve bit-for-bit identical output to the original literal).
pub(crate) fn pi() -> f64 {
    3.14159265358979323846
}

/// `e()` — same constant as the C source.
pub(crate) fn e() -> f64 {
    2.71828182845904523536
}

/// `fac(a)` — factorial. Ported line-for-line from the C `fac()` function,
/// including its overflow behavior (returns `INFINITY` rather than panicking
/// or wrapping) and its domain check (`NAN` for negative input).
///
/// The original uses `unsigned int` for the loop counter bound check against
/// `UINT_MAX`, and `unsigned long int` (64-bit on the reference platforms)
/// for the accumulator bound-checked against `ULONG_MAX`. We use `u32`/`u64`
/// to match those widths exactly on a standard 64-bit target.
pub(crate) fn fac(a: f64) -> f64 {
    if a < 0.0 {
        return f64::NAN;
    }
    if a > u32::MAX as f64 {
        return f64::INFINITY;
    }
    let ua = a as u32;
    let mut result: u64 = 1;
    let mut i: u64 = 1;
    while i <= ua as u64 {
        if i > u64::MAX / result {
            return f64::INFINITY;
        }
        result *= i;
        i += 1;
    }
    result as f64
}

/// `ncr(n, r)` — n-choose-r. Ported line-for-line from the C `ncr()`.
pub(crate) fn ncr(n: f64, r: f64) -> f64 {
    if n < 0.0 || r < 0.0 || n < r {
        return f64::NAN;
    }
    if n > u32::MAX as f64 || r > u32::MAX as f64 {
        return f64::INFINITY;
    }
    let un = n as u32 as u64;
    let mut ur = r as u32 as u64;
    if ur > un / 2 {
        ur = un - ur;
    }
    let mut result: u64 = 1;
    let mut i: u64 = 1;
    while i <= ur {
        if result > u64::MAX / (un - ur + i) {
            return f64::INFINITY;
        }
        result *= un - ur + i;
        result /= i;
        i += 1;
    }
    result as f64
}

/// `npr(n, r)` — n-permute-r. Ported line-for-line: `ncr(n, r) * fac(r)`.
pub(crate) fn npr(n: f64, r: f64) -> f64 {
    ncr(n, r) * fac(r)
}

// The following thin wrappers exist only because Rust's `f64` methods
// (`x.cos()`) can't be named directly as free-standing `fn` items the way
// C's `cos(x)` from <math.h> can. Each one calls the exact same libm
// operation the C table referenced.
fn abs_(a: f64) -> f64 {
    a.abs()
}
fn acos_(a: f64) -> f64 {
    a.acos()
}
fn asin_(a: f64) -> f64 {
    a.asin()
}
fn atan_(a: f64) -> f64 {
    a.atan()
}
fn atan2_(a: f64, b: f64) -> f64 {
    a.atan2(b)
}
fn ceil_(a: f64) -> f64 {
    a.ceil()
}
fn cos_(a: f64) -> f64 {
    a.cos()
}
fn cosh_(a: f64) -> f64 {
    a.cosh()
}
fn exp_(a: f64) -> f64 {
    a.exp()
}
fn floor_(a: f64) -> f64 {
    a.floor()
}
fn ln_(a: f64) -> f64 {
    a.ln()
}
// NOTE: matches the C source's *default* compile-time option, i.e.
// `TE_NAT_LOG` is left undefined, so `log` means base-10 log, same as
// `log10`. (See the `pub const NAT_LOG` toggle below if you want the other
// behavior, matching `#define TE_NAT_LOG` in the C file.)
fn log_(a: f64) -> f64 {
    if NAT_LOG {
        a.ln()
    } else {
        a.log10()
    }
}
fn log10_(a: f64) -> f64 {
    a.log10()
}
fn pow_(a: f64, b: f64) -> f64 {
    a.powf(b)
}
fn sin_(a: f64) -> f64 {
    a.sin()
}
fn sinh_(a: f64) -> f64 {
    a.sinh()
}
fn sqrt_(a: f64) -> f64 {
    a.sqrt()
}
fn tan_(a: f64) -> f64 {
    a.tan()
}
fn tanh_(a: f64) -> f64 {
    a.tanh()
}

/// Compile-time option mirroring `#define TE_NAT_LOG` in tinyexpr.c.
/// `false` (the default, matching the shipped C source) makes `log` mean
/// base-10 logarithm. Set to `true` to make `log` mean natural logarithm.
pub const NAT_LOG: bool = false;

/// One entry of the builtin function table: (name, binding, is_pure).
/// All builtins are pure in the original (`TE_FLAG_PURE` is set on every
/// row), so `is_pure` is always `true` here, kept explicit for clarity and
/// parity with the original table's shape.
pub(crate) struct BuiltinEntry {
    pub name: &'static str,
    pub binding: Binding,
    pub is_pure: bool,
}

/// Must stay in alphabetical order by `name` -- `find_builtin` binary
/// searches this table, exactly like the original `functions[]` array.
pub(crate) static BUILTINS: &[BuiltinEntry] = &[
    BuiltinEntry { name: "abs", binding: Binding::Function1(abs_), is_pure: true },
    BuiltinEntry { name: "acos", binding: Binding::Function1(acos_), is_pure: true },
    BuiltinEntry { name: "asin", binding: Binding::Function1(asin_), is_pure: true },
    BuiltinEntry { name: "atan", binding: Binding::Function1(atan_), is_pure: true },
    BuiltinEntry { name: "atan2", binding: Binding::Function2(atan2_), is_pure: true },
    BuiltinEntry { name: "ceil", binding: Binding::Function1(ceil_), is_pure: true },
    BuiltinEntry { name: "cos", binding: Binding::Function1(cos_), is_pure: true },
    BuiltinEntry { name: "cosh", binding: Binding::Function1(cosh_), is_pure: true },
    BuiltinEntry { name: "e", binding: Binding::Function0(e), is_pure: true },
    BuiltinEntry { name: "exp", binding: Binding::Function1(exp_), is_pure: true },
    BuiltinEntry { name: "fac", binding: Binding::Function1(fac), is_pure: true },
    BuiltinEntry { name: "floor", binding: Binding::Function1(floor_), is_pure: true },
    BuiltinEntry { name: "ln", binding: Binding::Function1(ln_), is_pure: true },
    BuiltinEntry { name: "log", binding: Binding::Function1(log_), is_pure: true },
    BuiltinEntry { name: "log10", binding: Binding::Function1(log10_), is_pure: true },
    BuiltinEntry { name: "ncr", binding: Binding::Function2(ncr), is_pure: true },
    BuiltinEntry { name: "npr", binding: Binding::Function2(npr), is_pure: true },
    BuiltinEntry { name: "pi", binding: Binding::Function0(pi), is_pure: true },
    BuiltinEntry { name: "pow", binding: Binding::Function2(pow_), is_pure: true },
    BuiltinEntry { name: "sin", binding: Binding::Function1(sin_), is_pure: true },
    BuiltinEntry { name: "sinh", binding: Binding::Function1(sinh_), is_pure: true },
    BuiltinEntry { name: "sqrt", binding: Binding::Function1(sqrt_), is_pure: true },
    BuiltinEntry { name: "tan", binding: Binding::Function1(tan_), is_pure: true },
    BuiltinEntry { name: "tanh", binding: Binding::Function1(tanh_), is_pure: true },
];

/// Binary search over `BUILTINS`, a direct port of `find_builtin()`.
/// `name` is looked up by exact match against `BUILTINS[i].name`.
pub(crate) fn find_builtin(name: &str) -> Option<&'static BuiltinEntry> {
    // A plain binary_search_by is a faithful (and simpler/safer) stand-in
    // for the original's hand-rolled binary search over the same sorted,
    // NUL-terminated-array-shaped table.
    BUILTINS
        .binary_search_by(|entry| entry.name.cmp(name))
        .ok()
        .map(|i| &BUILTINS[i])
}
