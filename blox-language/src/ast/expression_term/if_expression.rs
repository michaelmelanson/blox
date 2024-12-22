use crate::ast::Block;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct If {
    pub condition: Box<Expression>,
    pub body: Block,
    pub elseif_branches: Vec<(Expression, Block)>,
    pub else_branch: Option<Block>,
}

impl std::fmt::Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.condition, self.body)?;

        for (condition, branch) in &self.elseif_branches {
            write!(f, " else if {} {}", condition, branch)?;
        }

        if let Some(branch) = &self.else_branch {
            write!(f, " else {}", branch)?;
        }

        Ok(())
    }
}
