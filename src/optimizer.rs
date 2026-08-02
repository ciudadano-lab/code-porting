use crate::ast::Expr;
use crate::evaluator::te_eval;
use crate::token::*;

/// Constant-fold pure sub-trees (exact original algorithm).
pub fn optimize(n: &mut Expr) {
    if n.type_ == TE_CONSTANT || n.type_ == TE_VARIABLE {
        return;
    }
    if is_pure(n.type_) {
        let ar = arity(n.type_) as usize;
        let mut known = true;
        for i in 0..ar {
            optimize(&mut n.parameters[i]);
            if n.parameters[i].type_ != TE_CONSTANT {
                known = false;
            }
        }
        if known {
            let value = unsafe { te_eval(n) };
            n.parameters.clear();
            n.type_ = TE_CONSTANT;
            n.value = value;
        }
    }
}