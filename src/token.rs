//! Token kinds used by the lexer / parser.
//! Mirrors the original C enums exactly.

pub const TOK_NULL: i32 = 24; // TE_CLOSURE7 + 1
pub const TOK_ERROR: i32 = 25;
pub const TOK_END: i32 = 26;
pub const TOK_SEP: i32 = 27;
pub const TOK_OPEN: i32 = 28;
pub const TOK_CLOSE: i32 = 29;
pub const TOK_NUMBER: i32 = 30;
pub const TOK_VARIABLE: i32 = 31;
pub const TOK_INFIX: i32 = 32;

// Expression node types (from tinyexpr.h)
pub const TE_VARIABLE: i32 = 0;
pub const TE_CONSTANT: i32 = 1;

pub const TE_FUNCTION0: i32 = 8;
pub const TE_FUNCTION1: i32 = 9;
pub const TE_FUNCTION2: i32 = 10;
pub const TE_FUNCTION3: i32 = 11;
pub const TE_FUNCTION4: i32 = 12;
pub const TE_FUNCTION5: i32 = 13;
pub const TE_FUNCTION6: i32 = 14;
pub const TE_FUNCTION7: i32 = 15;

pub const TE_CLOSURE0: i32 = 16;
pub const TE_CLOSURE1: i32 = 17;
pub const TE_CLOSURE2: i32 = 18;
pub const TE_CLOSURE3: i32 = 19;
pub const TE_CLOSURE4: i32 = 20;
pub const TE_CLOSURE5: i32 = 21;
pub const TE_CLOSURE6: i32 = 22;
pub const TE_CLOSURE7: i32 = 23;

pub const TE_FLAG_PURE: i32 = 32;

#[inline]
pub fn type_mask(t: i32) -> i32 {
    t & 0x0000_001F
}

#[inline]
pub fn is_pure(t: i32) -> bool {
    (t & TE_FLAG_PURE) != 0
}

#[inline]
pub fn is_function(t: i32) -> bool {
    (t & TE_FUNCTION0) != 0
}

#[inline]
pub fn is_closure(t: i32) -> bool {
    (t & TE_CLOSURE0) != 0
}

#[inline]
pub fn arity(t: i32) -> i32 {
    if (t & (TE_FUNCTION0 | TE_CLOSURE0)) != 0 {
        t & 0x0000_0007
    } else {
        0
    }
}