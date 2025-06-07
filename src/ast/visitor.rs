//! Generic visitor pattern implementation for the AST

use crate::ast::*;
use hecs::Entity;
use std::collections::HashSet;

#[cfg(test)]
#[path = "visitor_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "visitor_additional_tests.rs"]
mod additional_tests;

/// Traditional visitor trait with methods for each node type
pub trait Visitor {
    fn visit_module(&mut self, _ast: &Ast, _module: &Module) {}
    fn visit_function(&mut self, _ast: &Ast, _func: &Function) {}
    fn visit_let(&mut self, _ast: &Ast, _let_stmt: &Let) {}
    fn visit_return(&mut self, _ast: &Ast, _return_stmt: &Return) {}
    fn visit_expression(&mut self, _ast: &Ast, _expr: &Expression) {}
    fn visit_while(&mut self, _ast: &Ast, _while_stmt: &While) {}
    fn visit_for(&mut self, _ast: &Ast, _for_stmt: &For) {}
    fn visit_break(&mut self, _ast: &Ast, _break_stmt: &Break) {}
    fn visit_block(&mut self, _ast: &Ast, _block: &Block) {}
    fn visit_static(&mut self, _ast: &Ast, _static_stmt: &Static) {}
    fn visit_identifier(&mut self, _ast: &Ast, _ident: &Identifier) {}
    fn visit_literal(&mut self, _ast: &Ast, _literal: &Literal) {}
    fn visit_operation(&mut self, _ast: &Ast, _op: &Operation) {}
    fn visit_if(&mut self, _ast: &Ast, _if_expr: &If) {}
    fn visit_case(&mut self, _ast: &Ast, _case: &Case) {}
    fn visit_lambda(&mut self, _ast: &Ast, _lambda: &Lambda) {}
    fn visit_call(&mut self, _ast: &Ast, _call: &Call) {}
    fn visit_index(&mut self, _ast: &Ast, _index: &Index) {}
    fn visit_field(&mut self, _ast: &Ast, _field: &Field) {}

    // Literal visitors
    fn visit_integer_literal(&mut self, _ast: &Ast, _lit: &IntegerLiteral) {}
    fn visit_float_literal(&mut self, _ast: &Ast, _lit: &FloatLiteral) {}
    fn visit_string_literal(&mut self, _ast: &Ast, _lit: &StringLiteral) {}
    fn visit_boolean_literal(&mut self, _ast: &Ast, _lit: &BooleanLiteral) {}
    fn visit_array_literal(&mut self, _ast: &Ast, _lit: &ArrayLiteral) {}
    fn visit_null_literal(&mut self, _ast: &Ast, _lit: &NullLiteral) {}

    // Pattern visitors
    fn visit_tuple_pattern(&mut self, _ast: &Ast, _pattern: &TuplePattern) {}
    fn visit_struct_pattern(&mut self, _ast: &Ast, _pattern: &StructPattern) {}

    // Other visitors
    fn visit_parameter(&mut self, _ast: &Ast, _param: &Parameter) {}
    fn visit_if_branch(&mut self, _ast: &Ast, _branch: &IfBranch) {}
    fn visit_case_branch(&mut self, _ast: &Ast, _branch: &CaseBranch) {}
}

/// Generic component-based visitor that filters nodes by component presence
pub struct ComponentFilterVisitor<T> {
    component_type: std::marker::PhantomData<T>,
    pub visited_entities: HashSet<Entity>,
}

impl<T: 'static + Send + Sync> Default for ComponentFilterVisitor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + Send + Sync> ComponentFilterVisitor<T> {
    pub fn new() -> Self {
        Self {
            component_type: std::marker::PhantomData,
            visited_entities: HashSet::new(),
        }
    }

    /// Visit only nodes that have the specified component type
    pub fn visit_if_has_component<N: AstNode>(&mut self, ast: &Ast, node: &N) {
        let entity = node.entity_id();
        if ast.get_component::<T>(entity).is_some() {
            self.visited_entities.insert(entity);
        }
    }
}

/// Walk a statement and call visitor methods
pub fn walk_statement<V: Visitor>(visitor: &mut V, ast: &Ast, stmt: &Statement) {
    match stmt {
        Statement::Module(module) => {
            visitor.visit_module(ast, module);
            for stmt in &module.statements {
                walk_statement(visitor, ast, stmt);
            }
        }
        Statement::Function(func) => {
            visitor.visit_function(ast, func);
            for param in &func.parameters {
                visitor.visit_parameter(ast, param);
                walk_expression(visitor, ast, &param.type_expr);
            }
            if let Some(return_type) = &func.return_type {
                walk_expression(visitor, ast, return_type);
            }
            if let Some(body) = &func.body {
                walk_block(visitor, ast, body);
            }
        }
        Statement::Let(let_stmt) => {
            visitor.visit_let(ast, let_stmt);
            walk_pattern(visitor, ast, &let_stmt.pattern);
            if let Some(value) = &let_stmt.value {
                walk_expression(visitor, ast, value);
            }
            if let Some(type_expr) = &let_stmt.type_expr {
                walk_expression(visitor, ast, type_expr);
            }
        }
        Statement::Return(return_stmt) => {
            visitor.visit_return(ast, return_stmt);
            if let Some(value) = &return_stmt.value {
                walk_expression(visitor, ast, value);
            }
        }
        Statement::Expression(expr) => {
            walk_expression(visitor, ast, expr);
        }
        Statement::While(while_stmt) => {
            visitor.visit_while(ast, while_stmt);
            walk_expression(visitor, ast, &while_stmt.condition);
            walk_block(visitor, ast, &while_stmt.body);
        }
        Statement::For(for_stmt) => {
            visitor.visit_for(ast, for_stmt);
            if let Some(init) = &for_stmt.init {
                walk_statement(visitor, ast, init);
            }
            if let Some(condition) = &for_stmt.condition {
                walk_expression(visitor, ast, condition);
            }
            if let Some(update) = &for_stmt.update {
                walk_expression(visitor, ast, update);
            }
            walk_block(visitor, ast, &for_stmt.body);
        }
        Statement::Break(break_stmt) => {
            visitor.visit_break(ast, break_stmt);
            if let Some(value) = &break_stmt.value {
                walk_expression(visitor, ast, value);
            }
        }
        Statement::Block(block) => {
            walk_block(visitor, ast, block);
        }
        Statement::Static(static_stmt) => {
            visitor.visit_static(ast, static_stmt);
            for stmt in &static_stmt.statements {
                walk_statement(visitor, ast, stmt);
            }
        }
    }
}

/// Walk an expression and call visitor methods
pub fn walk_expression<V: Visitor>(visitor: &mut V, ast: &Ast, expr: &Expression) {
    visitor.visit_expression(ast, expr);

    match expr {
        Expression::Identifier(ident) => {
            visitor.visit_identifier(ast, ident);
        }
        Expression::Literal(lit) => {
            visitor.visit_literal(ast, lit);
            walk_literal(visitor, ast, lit);
        }
        Expression::Operation(op) => {
            visitor.visit_operation(ast, op);
            for element in &op.elements {
                if let OperationElement::Operand(operand) = element {
                    walk_expression(visitor, ast, operand);
                }
            }
        }
        Expression::If(if_expr) => {
            visitor.visit_if(ast, if_expr);
            for branch in &if_expr.branches {
                visitor.visit_if_branch(ast, branch);
                walk_expression(visitor, ast, &branch.condition);
                walk_expression(visitor, ast, &branch.body);
            }
            if let Some(else_branch) = &if_expr.else_branch {
                walk_expression(visitor, ast, else_branch);
            }
        }
        Expression::Case(case) => {
            visitor.visit_case(ast, case);
            walk_expression(visitor, ast, &case.value);
            for branch in &case.branches {
                visitor.visit_case_branch(ast, branch);
                walk_pattern(visitor, ast, &branch.pattern);
                walk_expression(visitor, ast, &branch.body);
                if let Some(guard) = &branch.guard {
                    walk_expression(visitor, ast, guard);
                }
            }
        }
        Expression::Lambda(lambda) => {
            visitor.visit_lambda(ast, lambda);
            for param in &lambda.parameters {
                visitor.visit_parameter(ast, param);
                walk_expression(visitor, ast, &param.type_expr);
            }
            if let Some(return_type) = &lambda.return_type {
                walk_expression(visitor, ast, return_type);
            }
            walk_expression(visitor, ast, &lambda.body);
        }
        Expression::Call(call) => {
            visitor.visit_call(ast, call);
            walk_expression(visitor, ast, &call.function);
            for arg in &call.arguments {
                walk_expression(visitor, ast, arg);
            }
        }
        Expression::Index(index) => {
            visitor.visit_index(ast, index);
            walk_expression(visitor, ast, &index.object);
            walk_expression(visitor, ast, &index.index);
        }
        Expression::Field(field) => {
            visitor.visit_field(ast, field);
            walk_expression(visitor, ast, &field.object);
        }
        Expression::Block(block) => {
            walk_block(visitor, ast, block);
        }
        Expression::Grouped(grouped) => {
            walk_expression(visitor, ast, grouped);
        }
    }
}

/// Walk a literal
pub fn walk_literal<V: Visitor>(visitor: &mut V, ast: &Ast, literal: &Literal) {
    match literal {
        Literal::Integer(i) => visitor.visit_integer_literal(ast, i),
        Literal::Float(f) => visitor.visit_float_literal(ast, f),
        Literal::String(s) => {
            visitor.visit_string_literal(ast, s);
            walk_string_literal(visitor, ast, s);
        }
        Literal::Boolean(b) => visitor.visit_boolean_literal(ast, b),
        Literal::Array(a) => {
            visitor.visit_array_literal(ast, a);
            walk_array_literal(visitor, ast, a);
        }
        Literal::Null(n) => visitor.visit_null_literal(ast, n),
    }
}

/// Walk a string literal
pub fn walk_string_literal<V: Visitor>(visitor: &mut V, ast: &Ast, string: &StringLiteral) {
    for fragment in &string.fragments {
        if let StringFragment::Interpolation(expr) = fragment {
            walk_expression(visitor, ast, expr);
        }
    }
}

/// Walk an array literal
pub fn walk_array_literal<V: Visitor>(visitor: &mut V, ast: &Ast, array: &ArrayLiteral) {
    for element in &array.elements {
        walk_expression(visitor, ast, element);
    }
}

/// Walk a block and call visitor methods
pub fn walk_block<V: Visitor>(visitor: &mut V, ast: &Ast, block: &Block) {
    visitor.visit_block(ast, block);
    for stmt in &block.statements {
        walk_statement(visitor, ast, stmt);
    }
}

/// Walk the entire AST starting from the root
pub fn walk_ast<V: Visitor>(visitor: &mut V, ast: &Ast) {
    for stmt in &ast.root.statements {
        walk_statement(visitor, ast, stmt);
    }
}

/// Walk a pattern and call visitor methods
pub fn walk_pattern<V: Visitor>(visitor: &mut V, ast: &Ast, pattern: &Pattern) {
    match pattern {
        Pattern::Identifier(ident) => {
            visitor.visit_identifier(ast, ident);
        }
        Pattern::Tuple(tuple) => {
            visitor.visit_tuple_pattern(ast, tuple);
            for pattern in &tuple.patterns {
                walk_pattern(visitor, ast, pattern);
            }
        }
        Pattern::Struct(struct_pattern) => {
            visitor.visit_struct_pattern(ast, struct_pattern);
            for (_, pattern) in &struct_pattern.fields {
                walk_pattern(visitor, ast, pattern);
            }
        }
    }
}

/// Generic traversal for any node implementing Traversable
pub fn traverse_generic<C, V: GenericVisitor<C>, N: Traversable<C> + AstNode>(
    visitor: &mut V,
    ast: &Ast<C>,
    node: &N,
) {
    node.accept(visitor, ast);
}

/// Filter-based traversal that visits only nodes with specific components
pub struct FilterTraverser<T> {
    component_type: std::marker::PhantomData<T>,
}

impl<T: 'static + Send + Sync> Default for FilterTraverser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + Send + Sync> FilterTraverser<T> {
    pub fn new() -> Self {
        Self {
            component_type: std::marker::PhantomData,
        }
    }

    pub fn traverse<F>(&self, ast: &Ast, mut visit_fn: F)
    where
        F: FnMut(Entity, &T),
    {
        for (entity, component) in ast.world.query::<&T>().iter() {
            visit_fn(entity, component);
        }
    }

    pub fn traverse_mut<F>(&self, ast: &mut Ast, mut visit_fn: F)
    where
        F: FnMut(Entity, &mut T),
    {
        for (entity, component) in ast.world.query::<&mut T>().iter() {
            visit_fn(entity, component);
        }
    }
}

/// Component filtering wrapper for any visitor
pub struct ComponentFilterWrapper<V, F> {
    pub inner: V,
    pub filter: F,
}

impl<V, F> ComponentFilterWrapper<V, F>
where
    V: Visitor,
    F: Fn(&Ast, Entity) -> bool,
{
    pub fn new(inner: V, filter: F) -> Self {
        Self { inner, filter }
    }
}

impl<V, F> Visitor for ComponentFilterWrapper<V, F>
where
    V: Visitor,
    F: Fn(&Ast, Entity) -> bool,
{
    fn visit_module(&mut self, ast: &Ast, module: &Module) {
        if (self.filter)(ast, module.entity) {
            self.inner.visit_module(ast, module);
        }
    }

    fn visit_function(&mut self, ast: &Ast, func: &Function) {
        if (self.filter)(ast, func.entity) {
            self.inner.visit_function(ast, func);
        }
    }

    fn visit_let(&mut self, ast: &Ast, let_stmt: &Let) {
        if (self.filter)(ast, let_stmt.entity) {
            self.inner.visit_let(ast, let_stmt);
        }
    }

    fn visit_return(&mut self, ast: &Ast, ret: &Return) {
        if (self.filter)(ast, ret.entity) {
            self.inner.visit_return(ast, ret);
        }
    }

    fn visit_while(&mut self, ast: &Ast, while_stmt: &While) {
        if (self.filter)(ast, while_stmt.entity) {
            self.inner.visit_while(ast, while_stmt);
        }
    }

    fn visit_for(&mut self, ast: &Ast, for_stmt: &For) {
        if (self.filter)(ast, for_stmt.entity) {
            self.inner.visit_for(ast, for_stmt);
        }
    }

    fn visit_break(&mut self, ast: &Ast, break_stmt: &Break) {
        if (self.filter)(ast, break_stmt.entity) {
            self.inner.visit_break(ast, break_stmt);
        }
    }

    fn visit_block(&mut self, ast: &Ast, block: &Block) {
        if (self.filter)(ast, block.entity) {
            self.inner.visit_block(ast, block);
        }
    }

    fn visit_static(&mut self, ast: &Ast, static_stmt: &Static) {
        if (self.filter)(ast, static_stmt.entity) {
            self.inner.visit_static(ast, static_stmt);
        }
    }

    fn visit_if(&mut self, ast: &Ast, if_expr: &If) {
        if (self.filter)(ast, if_expr.entity) {
            self.inner.visit_if(ast, if_expr);
        }
    }

    fn visit_case(&mut self, ast: &Ast, case: &Case) {
        if (self.filter)(ast, case.entity) {
            self.inner.visit_case(ast, case);
        }
    }

    fn visit_lambda(&mut self, ast: &Ast, lambda: &Lambda) {
        if (self.filter)(ast, lambda.entity) {
            self.inner.visit_lambda(ast, lambda);
        }
    }

    fn visit_operation(&mut self, ast: &Ast, op: &Operation) {
        if (self.filter)(ast, op.entity) {
            self.inner.visit_operation(ast, op);
        }
    }

    fn visit_call(&mut self, ast: &Ast, call: &Call) {
        if (self.filter)(ast, call.entity) {
            self.inner.visit_call(ast, call);
        }
    }

    fn visit_index(&mut self, ast: &Ast, index: &Index) {
        if (self.filter)(ast, index.entity) {
            self.inner.visit_index(ast, index);
        }
    }

    fn visit_field(&mut self, ast: &Ast, field: &Field) {
        if (self.filter)(ast, field.entity) {
            self.inner.visit_field(ast, field);
        }
    }

    fn visit_identifier(&mut self, ast: &Ast, ident: &Identifier) {
        if (self.filter)(ast, ident.entity) {
            self.inner.visit_identifier(ast, ident);
        }
    }

    fn visit_integer_literal(&mut self, ast: &Ast, lit: &IntegerLiteral) {
        if (self.filter)(ast, lit.entity) {
            self.inner.visit_integer_literal(ast, lit);
        }
    }

    fn visit_float_literal(&mut self, ast: &Ast, lit: &FloatLiteral) {
        if (self.filter)(ast, lit.entity) {
            self.inner.visit_float_literal(ast, lit);
        }
    }

    fn visit_string_literal(&mut self, ast: &Ast, lit: &StringLiteral) {
        if (self.filter)(ast, lit.entity) {
            self.inner.visit_string_literal(ast, lit);
        }
    }

    fn visit_boolean_literal(&mut self, ast: &Ast, lit: &BooleanLiteral) {
        if (self.filter)(ast, lit.entity) {
            self.inner.visit_boolean_literal(ast, lit);
        }
    }

    fn visit_array_literal(&mut self, ast: &Ast, lit: &ArrayLiteral) {
        if (self.filter)(ast, lit.entity) {
            self.inner.visit_array_literal(ast, lit);
        }
    }

    fn visit_null_literal(&mut self, ast: &Ast, lit: &NullLiteral) {
        if (self.filter)(ast, lit.entity) {
            self.inner.visit_null_literal(ast, lit);
        }
    }

    fn visit_tuple_pattern(&mut self, ast: &Ast, pattern: &TuplePattern) {
        if (self.filter)(ast, pattern.entity) {
            self.inner.visit_tuple_pattern(ast, pattern);
        }
    }

    fn visit_struct_pattern(&mut self, ast: &Ast, pattern: &StructPattern) {
        if (self.filter)(ast, pattern.entity) {
            self.inner.visit_struct_pattern(ast, pattern);
        }
    }

    fn visit_parameter(&mut self, ast: &Ast, param: &Parameter) {
        if (self.filter)(ast, param.entity) {
            self.inner.visit_parameter(ast, param);
        }
    }

    fn visit_if_branch(&mut self, ast: &Ast, branch: &IfBranch) {
        if (self.filter)(ast, branch.entity) {
            self.inner.visit_if_branch(ast, branch);
        }
    }

    fn visit_case_branch(&mut self, ast: &Ast, branch: &CaseBranch) {
        if (self.filter)(ast, branch.entity) {
            self.inner.visit_case_branch(ast, branch);
        }
    }
}
