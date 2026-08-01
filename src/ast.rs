//! The expression tree (`te_expr` in the original) plus evaluation,
//! constant folding (`optimize()`), and debug printing (`te_print()`/`pn()`).

use std::fmt;
use std::os::raw::c_void;

/// Mirrors the `address`/`type` union+flags of the original `te_variable`.
/// Rather than packing an arity + "is it a closure" bit into an integer
/// (`TE_FUNCTION0..7`, `TE_CLOSURE0..7`) and reinterpreting a `const void*`
/// through a cast at call time, each shape gets its own variant with a
/// correctly-typed `fn` pointer. The set of shapes and their arities (0-7,
/// plain or "closure with a context pointer") is unchanged from the
/// original -- only the *representation* is safer.
#[derive(Clone, Copy)]
pub enum Binding {
    /// A variable bound to the address of an external `f64`, exactly like
    /// `TE_VARIABLE` in the C version: the pointee is re-read on every
    /// `eval()`, so the caller can mutate it between evaluations without
    /// recompiling the expression.
    Variable(*const f64),

    Function0(fn() -> f64),
    Function1(fn(f64) -> f64),
    Function2(fn(f64, f64) -> f64),
    Function3(fn(f64, f64, f64) -> f64),
    Function4(fn(f64, f64, f64, f64) -> f64),
    Function5(fn(f64, f64, f64, f64, f64) -> f64),
    Function6(fn(f64, f64, f64, f64, f64, f64) -> f64),
    Function7(fn(f64, f64, f64, f64, f64, f64, f64) -> f64),

    /// `TE_CLOSUREn`: like `FunctionN` but the function additionally
    /// receives an opaque `context` pointer as its first argument.
    Closure0(fn(*mut c_void) -> f64),
    Closure1(fn(*mut c_void, f64) -> f64),
    Closure2(fn(*mut c_void, f64, f64) -> f64),
    Closure3(fn(*mut c_void, f64, f64, f64) -> f64),
    Closure4(fn(*mut c_void, f64, f64, f64, f64) -> f64),
    Closure5(fn(*mut c_void, f64, f64, f64, f64, f64) -> f64),
    Closure6(fn(*mut c_void, f64, f64, f64, f64, f64, f64) -> f64),
    Closure7(fn(*mut c_void, f64, f64, f64, f64, f64, f64, f64) -> f64),
}

// SAFETY: `Binding` only ever holds `fn` pointers (always `Send + Sync`) or
// a raw `*const f64` address supplied by the caller when binding a
// `Variable`. The pointer itself is never mutated through this type, only
// dereferenced for reads in `eval()`, exactly mirroring how the original
// C library treats `te_variable::address` and the static `functions[]`
// table as freely shareable read-only data.
unsafe impl Sync for Binding {}
unsafe impl Send for Binding {}

impl Binding {
    /// Equivalent of the `ARITY(TYPE)` macro.
    pub(crate) fn arity(&self) -> usize {
        use Binding::*;
        match self {
            Variable(_) => 0,
            Function0(_) | Closure0(_) => 0,
            Function1(_) | Closure1(_) => 1,
            Function2(_) | Closure2(_) => 2,
            Function3(_) | Closure3(_) => 3,
            Function4(_) | Closure4(_) => 4,
            Function5(_) | Closure5(_) => 5,
            Function6(_) | Closure6(_) => 6,
            Function7(_) | Closure7(_) => 7,
        }
    }

    /// Equivalent of `TYPE_MASK(type) == TE_VARIABLE`.
    pub(crate) fn is_variable(&self) -> bool {
        matches!(self, Binding::Variable(_))
    }
}

/// The node kind, equivalent to what `TYPE_MASK(n->type)` distinguishes in
/// the original: a constant (`TE_CONSTANT`), a bound variable
/// (`TE_VARIABLE`), or a function/closure call of some arity
/// (`TE_FUNCTIONn` / `TE_CLOSUREn`).
pub(crate) enum ExprKind {
    Constant(f64),
    Variable(*const f64),
    Call {
        binding: Binding,
        /// Equivalent of `TE_FLAG_PURE`: true if `optimize()` is allowed to
        /// constant-fold a call to this function when every argument is
        /// itself constant.
        is_pure: bool,
        /// Equivalent of the extra closure parameter slot; unused
        /// (null) for non-closures.
        context: *mut c_void,
        /// Equivalent of `n->parameters[0..arity]`.
        args: Vec<Expr>,
    },
}

/// A compiled expression tree. Equivalent of `te_expr *`.
///
/// There is no explicit `te_free` in this port: dropping an `Expr` (or
/// letting it go out of scope) recursively frees the whole tree via `Vec`'s
/// and `Box`'s ordinary `Drop` behavior, which is exactly what
/// `te_free`/`te_free_parameters` did by hand in C. A `te_free` free
/// function is still provided for call-site parity; it is a no-op that just
/// takes ownership and drops.
pub struct Expr {
    pub(crate) kind: ExprKind,
}

impl Expr {
    pub(crate) fn constant(value: f64) -> Self {
        Expr { kind: ExprKind::Constant(value) }
    }

    pub(crate) fn variable(address: *const f64) -> Self {
        Expr { kind: ExprKind::Variable(address) }
    }

    pub(crate) fn call(binding: Binding, is_pure: bool, context: *mut c_void, args: Vec<Expr>) -> Self {
        Expr { kind: ExprKind::Call { binding, is_pure, context, args } }
    }

    /// Port of `te_eval()`.
    ///
    /// # Safety / behavior parity
    /// Reading a `Variable` binding dereferences a raw pointer supplied by
    /// the caller when compiling the expression -- exactly as the original
    /// does by dereferencing `n->bound`. The caller is responsible for
    /// ensuring that pointer stays valid for as long as the `Expr` is used,
    /// same as in the C API.
    pub fn eval(&self) -> f64 {
        match &self.kind {
            ExprKind::Constant(v) => *v,
            ExprKind::Variable(p) => unsafe { **p },
            ExprKind::Call { binding, context, args, .. } => {
                // Evaluate every argument first, left to right -- same
                // order as the M(0), M(1), ... expansions in te_eval().
                let vals: Vec<f64> = args.iter().map(Expr::eval).collect();
                call_binding(binding, *context, &vals)
            }
        }
    }

    /// Port of `optimize()`: evaluates as much of the tree as possible at
    /// compile time. Deliberately preserves the original's exact
    /// (slightly quirky) behavior: a node's children are only visited if
    /// the node itself is a pure call. An impure call's subtree is left
    /// completely unoptimized, exactly like the C version.
    pub(crate) fn optimize(&mut self) {
        match &mut self.kind {
            ExprKind::Constant(_) => {}
            ExprKind::Variable(_) => {}
            ExprKind::Call { is_pure, args, .. } => {
                if *is_pure {
                    let mut known = true;
                    for a in args.iter_mut() {
                        a.optimize();
                        if !matches!(a.kind, ExprKind::Constant(_)) {
                            known = false;
                        }
                    }
                    if known {
                        let value = self.eval();
                        self.kind = ExprKind::Constant(value);
                    }
                }
            }
        }
    }

    /// True if this node is a folded-down constant, i.e. what `ex->value`
    /// would validly give you without calling `te_eval` in the original
    /// (only meaningful to check right after `te_compile`, before any
    /// further tree surgery).
    pub fn is_constant(&self) -> bool {
        matches!(self.kind, ExprKind::Constant(_))
    }

    /// Port of `te_print()`/`pn()`: dumps the tree structure for
    /// debugging. Pointer values are printed instead of being
    /// meaningful across runs, same as the original (which prints
    /// `n->bound` and `n->parameters[i]` raw addresses).
    pub fn print(&self) {
        self.print_at(0);
    }

    fn print_at(&self, depth: usize) {
        print!("{:width$}", "", width = depth);
        match &self.kind {
            ExprKind::Constant(v) => println!("{:.6}", v),
            ExprKind::Variable(p) => println!("bound {:p}", *p),
            ExprKind::Call { binding, args, .. } => {
                print!("f{}", binding.arity());
                for a in args {
                    print!(" {:p}", a);
                }
                println!();
                for a in args {
                    a.print_at(depth + 1);
                }
            }
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Constant(v) => write!(f, "Constant({v})"),
            ExprKind::Variable(p) => write!(f, "Variable({p:p})"),
            ExprKind::Call { binding, args, .. } => {
                write!(f, "Call(arity={}, args={:?})", binding.arity(), args)
            }
        }
    }
}

/// Port of the `TE_FUN(...)`-cast-and-call dispatch inside `te_eval()`'s
/// big switch. `vals.len()` always equals `binding.arity()` by
/// construction (the parser only ever builds a `Call` with exactly that
/// many argument slots), so the indexing below can't go out of bounds.
fn call_binding(binding: &Binding, context: *mut c_void, vals: &[f64]) -> f64 {
    use Binding::*;
    match *binding {
        Variable(_) => f64::NAN, // unreachable: Variable is its own ExprKind, never wrapped in Call
        Function0(f) => f(),
        Function1(f) => f(vals[0]),
        Function2(f) => f(vals[0], vals[1]),
        Function3(f) => f(vals[0], vals[1], vals[2]),
        Function4(f) => f(vals[0], vals[1], vals[2], vals[3]),
        Function5(f) => f(vals[0], vals[1], vals[2], vals[3], vals[4]),
        Function6(f) => f(vals[0], vals[1], vals[2], vals[3], vals[4], vals[5]),
        Function7(f) => f(vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6]),
        Closure0(f) => f(context),
        Closure1(f) => f(context, vals[0]),
        Closure2(f) => f(context, vals[0], vals[1]),
        Closure3(f) => f(context, vals[0], vals[1], vals[2]),
        Closure4(f) => f(context, vals[0], vals[1], vals[2], vals[3]),
        Closure5(f) => f(context, vals[0], vals[1], vals[2], vals[3], vals[4]),
        Closure6(f) => f(context, vals[0], vals[1], vals[2], vals[3], vals[4], vals[5]),
        Closure7(f) => f(context, vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6]),
    }
}

/// A variable or function binding supplied at compile time -- port of
/// `te_variable`.
#[derive(Clone, Copy)]
pub struct Variable {
    pub name: &'static str,
    pub binding: Binding,
    /// Equivalent of the `TE_FLAG_PURE` bit. Ignored for plain variables.
    pub is_pure: bool,
    /// Equivalent of the `context` field; only read for `Closure*` bindings.
    pub context: *mut c_void,
}

impl Variable {
    pub fn variable(name: &'static str, address: *const f64) -> Self {
        Variable { name, binding: Binding::Variable(address), is_pure: false, context: std::ptr::null_mut() }
    }

    pub fn function0(name: &'static str, f: fn() -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function0(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function1(name: &'static str, f: fn(f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function1(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function2(name: &'static str, f: fn(f64, f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function2(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function3(name: &'static str, f: fn(f64, f64, f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function3(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function4(name: &'static str, f: fn(f64, f64, f64, f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function4(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function5(name: &'static str, f: fn(f64, f64, f64, f64, f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function5(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function6(name: &'static str, f: fn(f64, f64, f64, f64, f64, f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function6(f), is_pure, context: std::ptr::null_mut() }
    }
    pub fn function7(name: &'static str, f: fn(f64, f64, f64, f64, f64, f64, f64) -> f64, is_pure: bool) -> Self {
        Variable { name, binding: Binding::Function7(f), is_pure, context: std::ptr::null_mut() }
    }

    pub fn closure0(name: &'static str, f: fn(*mut c_void) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure0(f), is_pure, context }
    }
    pub fn closure1(name: &'static str, f: fn(*mut c_void, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure1(f), is_pure, context }
    }
    pub fn closure2(name: &'static str, f: fn(*mut c_void, f64, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure2(f), is_pure, context }
    }
    pub fn closure3(name: &'static str, f: fn(*mut c_void, f64, f64, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure3(f), is_pure, context }
    }
    pub fn closure4(name: &'static str, f: fn(*mut c_void, f64, f64, f64, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure4(f), is_pure, context }
    }
    pub fn closure5(name: &'static str, f: fn(*mut c_void, f64, f64, f64, f64, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure5(f), is_pure, context }
    }
    pub fn closure6(name: &'static str, f: fn(*mut c_void, f64, f64, f64, f64, f64, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure6(f), is_pure, context }
    }
    pub fn closure7(name: &'static str, f: fn(*mut c_void, f64, f64, f64, f64, f64, f64, f64) -> f64, is_pure: bool, context: *mut c_void) -> Self {
        Variable { name, binding: Binding::Closure7(f), is_pure, context }
    }
}
