use super::*;
use serde::{Deserialize, Serialize};

/// IfExpression represents an if expression in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfExpression {
    pub pos: Position,
    pub if_branch: Option<IfBranch>,
    pub else_ifs: Vec<IfBranch>,
    pub else_branch: Option<Box<ExpressionKind>>,
}

impl SyntaxNode for IfExpression {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        if let Some(if_branch) = &self.if_branch {
            children.push(if_branch as &dyn SyntaxNode);
        }
        for else_if in &self.else_ifs {
            children.push(else_if as &dyn SyntaxNode);
        }
        if let Some(else_branch) = &self.else_branch {
            children.push(else_branch.as_ref() as &dyn SyntaxNode);
        }
        children
    }
}

/// IfBranch represents a branch in an if expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfBranch {
    pub pos: Position,
    pub condition: Box<ExpressionKind>,
    pub then: Box<ExpressionKind>,
}

impl SyntaxNode for IfBranch {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![
            self.condition.as_ref() as &dyn SyntaxNode,
            self.then.as_ref() as &dyn SyntaxNode,
        ]
    }
}

/// CaseExpression represents a case expression in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseExpression {
    pub pos: Position,
    pub branches: Vec<CaseBranch>,
}

impl SyntaxNode for CaseExpression {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.branches.iter().map(|b| b as &dyn SyntaxNode).collect()
    }
}

/// CaseBranch represents a branch in a case expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseBranch {
    pub pos: Position,
    pub pattern: Option<Box<ExpressionKind>>,
    pub then: Box<ExpressionKind>,
}

impl SyntaxNode for CaseBranch {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        if let Some(pattern) = &self.pattern {
            children.push(pattern.as_ref() as &dyn SyntaxNode);
        }
        children.push(self.then.as_ref() as &dyn SyntaxNode);
        children
    }
}

/// CaseArm is an alias for CaseBranch for compatibility
pub type CaseArm = CaseBranch;

/// Lambda represents a lambda expression in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lambda {
    pub pos: Position,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<Box<AtomKind>>,
    pub body: Block,
}

impl SyntaxNode for Lambda {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        for param in &self.parameters {
            children.push(param as &dyn SyntaxNode);
        }
        if let Some(return_type) = &self.return_type {
            children.push(return_type.as_ref() as &dyn SyntaxNode);
        }
        children.push(&self.body as &dyn SyntaxNode);
        children
    }
}

/// BlockExpression represents a block expression in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockExpression {
    pub pos: Position,
    pub is_static: bool,
    pub block: Block,
}

impl SyntaxNode for BlockExpression {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.block as &dyn SyntaxNode]
    }
}

/// TypeExpression represents a type used as an expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeExpression {
    pub pos: Position,
    pub type_kind: TypeKind,
}

impl SyntaxNode for TypeExpression {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.type_kind as &dyn SyntaxNode]
    }
}

/// Operation represents a sequence of operations on operands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub pos: Position,
    pub prefix: Vec<String>,
    pub operands: Vec<Operand>,
}

impl SyntaxNode for Operation {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.operands.iter().map(|o| o as &dyn SyntaxNode).collect()
    }
}

/// Operand represents a single operand in an operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operand {
    pub pos: Position,
    pub atom: AtomKind,
    pub postfix: Vec<Postfix>,
}

impl SyntaxNode for Operand {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        children.push(&self.atom as &dyn SyntaxNode);
        for postfix in &self.postfix {
            children.push(postfix as &dyn SyntaxNode);
        }
        children
    }
}

/// Postfix represents a postfix operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Postfix {
    pub pos: Position,
    pub invocation: Vec<ExpressionKind>,
    pub index: Option<Box<ExpressionKind>>,
    pub field: Option<String>,
    pub operator: Option<String>,
}

impl SyntaxNode for Postfix {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        for expr in &self.invocation {
            children.push(expr as &dyn SyntaxNode);
        }
        if let Some(index) = &self.index {
            children.push(index.as_ref() as &dyn SyntaxNode);
        }
        children
    }
}

impl SyntaxNode for ExpressionKind {
    fn position(&self) -> &Position {
        match self {
            ExpressionKind::If(i) => i.position(),
            ExpressionKind::Case(c) => c.position(),
            ExpressionKind::Lambda(l) => l.position(),
            ExpressionKind::Block(b) => b.position(),
            ExpressionKind::Type(t) => t.position(),
            ExpressionKind::Operation(o) => o.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            ExpressionKind::If(i) => i.children(),
            ExpressionKind::Case(c) => c.children(),
            ExpressionKind::Lambda(l) => l.children(),
            ExpressionKind::Block(b) => b.children(),
            ExpressionKind::Type(t) => t.children(),
            ExpressionKind::Operation(o) => o.children(),
        }
    }
}
