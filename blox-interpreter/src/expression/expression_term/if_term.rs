use blox_language::ast;

use crate::{
    expression::condition::evaluate_condition, program::evaluate_block, EvaluationContext,
    RuntimeError, Value,
};

pub fn evaluate_if_term(
    if_term: &ast::If,
    context: &mut EvaluationContext,
) -> Result<Value, RuntimeError> {
    let ast::If {
        condition,
        body,
        elseif_branches,
        else_branch,
    } = if_term;

    if evaluate_condition(condition, context)? {
        return evaluate_block(body, context);
    }

    for elseif_branch in elseif_branches {
        if evaluate_condition(&elseif_branch.0, context)? {
            return evaluate_block(&elseif_branch.1, context);
        }
    }

    if let Some(else_branch) = else_branch {
        return evaluate_block(else_branch, context);
    }

    Ok(Value::Void)
}
