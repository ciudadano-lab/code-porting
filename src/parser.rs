//! Lexer (`next_token`) and recursive-descent parser
//! (`base`/`power`/`factor`/`term`/`expr`/`list`), ported statement-by-statement
//! from tinyexpr.c. The grammar, precedence, associativity, and every
//! syntax-error condition are unchanged from the original:
//!
//! ```text
//! <list>      =    <expr> {"," <expr>}
//! <expr>      =    <term> {("+" | "-") <term>}
//! <term>      =    <factor> {("*" | "/" | "%") <factor>}
//! <factor>    =    <power> {"^" <power>}
//! <power>     =    {("-" | "+")} <base>
//! <base>      =    <constant> | <variable> | <function-0> {"(" ")"}
//!                   | <function-1> <power> | <function-X> "(" <expr> {"," <expr>} ")"
//!                   | "(" <list> ")"
//! ```

use crate::ast::{Binding, Expr, Variable};
use crate::builtins::find_builtin;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}

#[derive(Clone, Copy)]
enum Tok {
    End,
    Sep,
    Open,
    Close,
    Error,
    Number(f64),
    Variable(*const f64),
    Infix(InfixOp),
    Func(Binding, bool, *mut std::os::raw::c_void),
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic()
}
fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

struct Lexer<'a> {
    bytes: &'a [u8],
    pos: usize,
    lookup: &'a [Variable],
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str, lookup: &'a [Variable]) -> Self {
        Lexer { bytes: src.as_bytes(), pos: 0, lookup }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// Port of `next_token()`.
    fn next_token(&mut self) -> Tok {
        loop {
            let Some(c) = self.peek() else {
                return Tok::End;
            };

            // Try reading a number (matches `isdigit(c) || c == '.'`).
            if c.is_ascii_digit() || c == b'.' {
                return self.scan_number();
            }

            if is_ident_start(c) {
                let start = self.pos;
                while self.peek().map(is_ident_continue).unwrap_or(false) {
                    self.pos += 1;
                }
                let name = std::str::from_utf8(&self.bytes[start..self.pos]).unwrap_or("");

                // Custom variables/functions take priority over builtins,
                // exactly like `find_lookup` being tried before
                // `find_builtin` in the original.
                if let Some(v) = self.lookup.iter().find(|v| v.name == name) {
                    return self.token_for_variable(v);
                }
                if let Some(entry) = find_builtin(name) {
                    return Tok::Func(entry.binding, entry.is_pure, std::ptr::null_mut());
                }
                return Tok::Error;
            }

            // Operators / punctuation / whitespace.
            let c = self.bytes[self.pos];
            self.pos += 1;
            return match c {
                b'+' => Tok::Infix(InfixOp::Add),
                b'-' => Tok::Infix(InfixOp::Sub),
                b'*' => Tok::Infix(InfixOp::Mul),
                b'/' => Tok::Infix(InfixOp::Div),
                b'^' => Tok::Infix(InfixOp::Pow),
                b'%' => Tok::Infix(InfixOp::Mod),
                b'(' => Tok::Open,
                b')' => Tok::Close,
                b',' => Tok::Sep,
                b' ' | b'\t' | b'\n' | b'\r' => continue, // matches the do/while-on-TOK_NULL skip loop
                _ => Tok::Error,
            };
        }
    }

    fn token_for_variable(&self, v: &Variable) -> Tok {
        if v.binding.is_variable() {
            let Binding::Variable(p) = v.binding else { unreachable!() };
            Tok::Variable(p)
        } else {
            Tok::Func(v.binding, v.is_pure, v.context)
        }
    }

    /// Scans a numeric literal starting at `self.pos` (which points at a
    /// digit or `.`). Mirrors `strtod`'s grammar for the subset that can
    /// appear here: `digits [. digits] [(e|E) [+|-] digits]`.
    ///
    /// Divergence from the original: a bare "." with no digits on either
    /// side is not a valid number in C's strtod either (it reports "no
    /// conversion", leaving the read position unmoved) -- but the original
    /// C parser has no guard against that, and unconditionally treats
    /// "starts with '.'" as "is a number", which can loop forever
    /// re-scanning the same lone '.' forever on malformed input.  This port
    /// treats a digit-less "." as a lex error (consuming the '.') instead
    /// of hanging, which is a deliberate robustness improvement; it does
    /// not change the result for any input that has at least one digit.
    fn scan_number(&mut self) -> Tok {
        let start = self.pos;
        let mut saw_digit = false;

        while self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
            self.pos += 1;
            saw_digit = true;
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            while self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
                self.pos += 1;
                saw_digit = true;
            }
        }
        if !saw_digit {
            // Lone '.' (or similar) -- not a number. See doc comment above.
            return Tok::Error;
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            let mut j = self.pos + 1;
            if matches!(self.bytes.get(j), Some(b'+') | Some(b'-')) {
                j += 1;
            }
            if self.bytes.get(j).map(|b| b.is_ascii_digit()).unwrap_or(false) {
                j += 1;
                while self.bytes.get(j).map(|b| b.is_ascii_digit()).unwrap_or(false) {
                    j += 1;
                }
                self.pos = j;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).unwrap_or("");
        let value = text.parse::<f64>().unwrap_or(f64::NAN);
        Tok::Number(value)
    }
}

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    tok: Tok,
    /// Set once parsing hits a construct that doesn't fit the grammar.
    /// Equivalent of the various explicit `s->type = TOK_ERROR;` sites in
    /// the C source that occur while the *current* token is something
    /// other than a genuine lex error (e.g. an unexpected `)` or an
    /// unexpected end-of-input where a sub-expression was required).
    failed: bool,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(src: &'a str, lookup: &'a [Variable]) -> Self {
        let mut lexer = Lexer::new(src, lookup);
        let tok = lexer.next_token();
        Parser { lexer, tok, failed: false }
    }

    fn advance(&mut self) {
        self.tok = self.lexer.next_token();
    }

    fn fail(&mut self) {
        self.failed = true;
        self.tok = Tok::Error;
    }

    /// True once the whole input has been consumed without any grammar
    /// violation -- equivalent of `s.type == TOK_END` at the end of
    /// `te_compile`, combined with there having been no explicit
    /// `TOK_ERROR` assignment along the way.
    pub(crate) fn ok(&self) -> bool {
        !self.failed && matches!(self.tok, Tok::End)
    }

    /// Equivalent of `s.next - s.start` (with the `if (*error == 0) *error = 1;`
    /// special case) computed in `te_compile` once parsing has stopped.
    pub(crate) fn error_position(&self) -> usize {
        if self.lexer.pos == 0 {
            1
        } else {
            self.lexer.pos
        }
    }

    /// Port of `list()`.
    pub(crate) fn parse_list(&mut self) -> Expr {
        let mut ret = self.parse_expr();
        while matches!(self.tok, Tok::Sep) {
            self.advance();
            let e = self.parse_expr();
            ret = Expr::call(comma_binding(), true, std::ptr::null_mut(), vec![ret, e]);
        }
        ret
    }

    /// Port of `expr()`.
    fn parse_expr(&mut self) -> Expr {
        let mut ret = self.parse_term();
        loop {
            let op = match self.tok {
                Tok::Infix(op @ (InfixOp::Add | InfixOp::Sub)) => op,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_term();
            ret = Expr::call(op_binding(op), true, std::ptr::null_mut(), vec![ret, rhs]);
        }
        ret
    }

    /// Port of `term()`.
    fn parse_term(&mut self) -> Expr {
        let mut ret = self.parse_factor();
        loop {
            let op = match self.tok {
                Tok::Infix(op @ (InfixOp::Mul | InfixOp::Div | InfixOp::Mod)) => op,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_factor();
            ret = Expr::call(op_binding(op), true, std::ptr::null_mut(), vec![ret, rhs]);
        }
        ret
    }

    /// Port of `factor()` (the default, non-`TE_POW_FROM_RIGHT` variant --
    /// the one actually compiled by the shipped C source, since that macro
    /// is left `#undef`'d there).
    fn parse_factor(&mut self) -> Expr {
        let mut ret = self.parse_power();
        while matches!(self.tok, Tok::Infix(InfixOp::Pow)) {
            self.advance();
            let rhs = self.parse_power();
            ret = Expr::call(op_binding(InfixOp::Pow), true, std::ptr::null_mut(), vec![ret, rhs]);
        }
        ret
    }

    /// Port of `power()`.
    fn parse_power(&mut self) -> Expr {
        let mut sign = 1i32;
        loop {
            match self.tok {
                Tok::Infix(InfixOp::Add) => self.advance(),
                Tok::Infix(InfixOp::Sub) => {
                    sign = -sign;
                    self.advance();
                }
                _ => break,
            }
        }
        let b = self.parse_base();
        if sign == 1 {
            b
        } else {
            Expr::call(Binding::Function1(negate_), true, std::ptr::null_mut(), vec![b])
        }
    }

    /// Port of `base()`.
    fn parse_base(&mut self) -> Expr {
        match self.tok {
            Tok::Number(v) => {
                self.advance();
                Expr::constant(v)
            }
            Tok::Variable(p) => {
                self.advance();
                Expr::variable(p)
            }
            Tok::Func(binding, is_pure, context) if binding.arity() == 0 => {
                self.advance();
                // Optional, ignored "()" after a zero-arg function, exactly
                // like the original: `pi` and `pi()` are both valid.
                if matches!(self.tok, Tok::Open) {
                    self.advance();
                    if !matches!(self.tok, Tok::Close) {
                        self.fail();
                    } else {
                        self.advance();
                    }
                }
                Expr::call(binding, is_pure, context, Vec::new())
            }
            Tok::Func(binding, is_pure, context) if binding.arity() == 1 => {
                self.advance();
                // A 1-arg function's argument is parsed as a <power>, not
                // a full <expr> -- so it binds tighter than +/-/*// and
                // does not accept a parenthesized argument list, matching
                // the original grammar exactly (`sin -1`, `sin 2^2` work;
                // commas do not).
                let arg = self.parse_power();
                Expr::call(binding, is_pure, context, vec![arg])
            }
            Tok::Func(binding, is_pure, context) => {
                let arity = binding.arity();
                self.advance();
                if !matches!(self.tok, Tok::Open) {
                    self.fail();
                    return Expr::call(binding, is_pure, context, Vec::new());
                }
                let mut args = Vec::with_capacity(arity);
                let mut i = 0usize;
                while i < arity {
                    self.advance(); // consumes '(' on i==0, ',' afterwards
                    args.push(self.parse_expr());
                    if !matches!(self.tok, Tok::Sep) {
                        break;
                    }
                    i += 1;
                }
                if !matches!(self.tok, Tok::Close) || i != arity.saturating_sub(1) {
                    self.fail();
                } else {
                    self.advance();
                }
                Expr::call(binding, is_pure, context, args)
            }
            Tok::Open => {
                self.advance();
                let ret = self.parse_list();
                if !matches!(self.tok, Tok::Close) {
                    self.fail();
                } else {
                    self.advance();
                }
                ret
            }
            // Unexpected token where a primary expression was required
            // (includes premature end-of-input, a stray ')', a stray ',',
            // and genuine lex errors) -- matches the `default:` arm of
            // `base()`'s switch, which fabricates a NAN constant node and
            // marks the parse as failed without consuming the token.
            _ => {
                self.fail();
                Expr::constant(f64::NAN)
            }
        }
    }
}

fn op_binding(op: InfixOp) -> Binding {
    match op {
        InfixOp::Add => Binding::Function2(add_),
        InfixOp::Sub => Binding::Function2(sub_),
        InfixOp::Mul => Binding::Function2(mul_),
        InfixOp::Div => Binding::Function2(div_),
        InfixOp::Pow => Binding::Function2(pow_),
        InfixOp::Mod => Binding::Function2(fmod_),
    }
}
fn comma_binding() -> Binding {
    Binding::Function2(comma_)
}

fn add_(a: f64, b: f64) -> f64 {
    a + b
}
fn sub_(a: f64, b: f64) -> f64 {
    a - b
}
fn mul_(a: f64, b: f64) -> f64 {
    a * b
}
fn div_(a: f64, b: f64) -> f64 {
    a / b
}
fn pow_(a: f64, b: f64) -> f64 {
    a.powf(b)
}
fn fmod_(a: f64, b: f64) -> f64 {
    a % b
}
fn negate_(a: f64) -> f64 {
    -a
}
fn comma_(a: f64, b: f64) -> f64 {
    let _ = a;
    b
}
