use hecs::Entity;
use crate::ast::*;
use super::{AstWorld, builder::AstBuilder, components::*};
use pest::Span;

pub struct AstConverter;

impl AstConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn convert(&self, root: &Root) -> AstWorld {
        let mut ast_world = AstWorld::new();
        let mut builder = AstBuilder::from_ast_world(&mut ast_world);
        let root_entity = self.convert_root(&mut builder, root);
        ast_world.set_root(root_entity);
        ast_world
    }

    fn convert_root(&self, builder: &mut AstBuilder, root: &Root) -> Entity {
        let children: Vec<Entity> = root.statements.iter()
            .map(|stmt| self.convert_decorated_statement(builder, stmt))
            .collect();
        builder.create_root(children)
    }

    fn convert_decorated_statement(&self, builder: &mut AstBuilder, stmt: &DecoratedStatement) -> Entity {
        let comments: Vec<Entity> = stmt.comments.iter()
            .map(|comment| self.convert_comment(builder, comment))
            .collect();
        
        let annotations: Vec<Entity> = stmt.annotations.iter()
            .map(|annot| self.convert_annotation(builder, annot))
            .collect();
        
        let statement_entity = self.convert_statement(builder, &stmt.statement);
        
        let span = self.position_to_span(stmt.statement.position());
        
        builder.create_decorated_statement(&span, comments, annotations, statement_entity)
    }

    fn convert_comment(&self, builder: &mut AstBuilder, comment: &Comment) -> Entity {
        let span = self.position_to_span(&comment.pos);
        let text = comment.text.clone().unwrap_or_default();
        let is_doc = text.starts_with("///") || text.starts_with("//!");
        builder.create_comment(&span, text, is_doc)
    }

    fn convert_annotation(&self, builder: &mut AstBuilder, annotation: &Annotation) -> Entity {
        let span = self.position_to_span(&annotation.pos);
        let arguments: Vec<Entity> = annotation.arguments.iter()
            .map(|expr| self.convert_expression(builder, expr))
            .collect();
        builder.create_annotation(&span, annotation.name.clone(), arguments)
    }

    fn convert_statement(&self, builder: &mut AstBuilder, stmt: &StatementKind) -> Entity {
        match stmt {
            StatementKind::Module(module) => self.convert_module(builder, module),
            StatementKind::Function(func) => self.convert_function(builder, func),
            StatementKind::Return(ret) => self.convert_return(builder, ret),
            StatementKind::Let(let_stmt) => self.convert_let(builder, let_stmt),
            StatementKind::Assignment(assign) => self.convert_assignment(builder, assign),
            StatementKind::Break(brk) => self.convert_break(builder, brk),
            StatementKind::While(while_stmt) => self.convert_while(builder, while_stmt),
            StatementKind::For(for_stmt) => self.convert_for(builder, for_stmt),
            StatementKind::Struct(struct_stmt) => self.convert_struct(builder, struct_stmt),
            StatementKind::Enum(enum_stmt) => self.convert_enum(builder, enum_stmt),
            StatementKind::Interface(iface) => self.convert_interface(builder, iface),
            StatementKind::Impl(impl_stmt) => self.convert_impl(builder, impl_stmt),
            StatementKind::Expression(expr) => self.convert_expression_statement(builder, expr),
        }
    }

    fn convert_module(&self, builder: &mut AstBuilder, module: &Module) -> Entity {
        let span = self.position_to_span(&module.pos);
        let declarations: Vec<Entity> = module.body.statements.iter()
            .map(|stmt| self.convert_decorated_statement(builder, stmt))
            .collect();
        builder.create_module(&span, module.name.clone(), declarations)
    }

    fn convert_function(&self, builder: &mut AstBuilder, func: &FunctionStatement) -> Entity {
        let span = self.position_to_span(&func.pos);
        let parameters: Vec<Entity> = func.parameters.iter()
            .map(|param| self.convert_function_parameter(builder, param))
            .collect();
        
        let return_type = func.return_type.as_ref()
            .map(|atom| self.convert_atom(builder, atom));
        
        let body = Some(self.convert_block_to_block_expression(builder, &func.block));
        
        builder.create_function(&span, func.name.clone(), func.is_static, parameters, return_type, body)
    }

    fn convert_block_to_block_expression(&self, builder: &mut AstBuilder, block: &Block) -> Entity {
        let span = self.position_to_span(&block.pos);
        let statements: Vec<Entity> = block.statements.iter()
            .map(|stmt| self.convert_decorated_statement(builder, stmt))
            .collect();
        builder.create_block_expression(&span, None, false, statements)
    }

    fn convert_function_parameter(&self, builder: &mut AstBuilder, param: &FunctionParameter) -> Entity {
        let span = self.position_to_span(&param.pos);
        let type_entity = self.convert_atom(builder, &param.param_type);
        builder.create_function_parameter(&span, param.name.clone(), type_entity)
    }

    fn convert_return(&self, builder: &mut AstBuilder, ret: &ReturnStatement) -> Entity {
        let span = self.position_to_span(&ret.pos);
        let value = ret.value.as_ref()
            .map(|expr| self.convert_expression(builder, expr));
        builder.create_return_statement(&span, ret.label.clone(), value)
    }

    fn convert_let(&self, builder: &mut AstBuilder, let_stmt: &LetStatement) -> Entity {
        let span = self.position_to_span(&let_stmt.pos);
        let binding = builder.create_binding(&span, vec![let_stmt.name.clone()]);
        let type_ref = let_stmt.var_type.as_ref()
            .map(|atom| self.convert_atom(builder, atom));
        let value = let_stmt.value.as_ref()
            .map(|expr| self.convert_expression(builder, expr));
        
        builder.create_let_statement(&span, let_stmt.is_static, let_stmt.is_mutable, binding, type_ref, value)
    }

    fn convert_assignment(&self, builder: &mut AstBuilder, assign: &AssignmentStatement) -> Entity {
        let span = self.position_to_span(&assign.pos);
        let target = self.convert_expression(builder, &assign.target);
        let value = self.convert_expression(builder, &assign.value);
        builder.create_operation(&span, "=".to_string(), Some(target), Some(value), None)
    }

    fn convert_break(&self, builder: &mut AstBuilder, brk: &BreakStatement) -> Entity {
        let span = self.position_to_span(&brk.pos);
        let value = brk.value.as_ref()
            .map(|expr| self.convert_expression(builder, expr));
        builder.create_break_statement(&span, brk.label.clone(), value)
    }

    fn convert_while(&self, builder: &mut AstBuilder, while_stmt: &WhileStatement) -> Entity {
        let span = self.position_to_span(&while_stmt.pos);
        let condition = self.convert_expression(builder, &while_stmt.condition);
        let body = self.convert_block_to_block_expression(builder, &while_stmt.body);
        builder.create_while_statement(&span, condition, body)
    }

    fn convert_for(&self, builder: &mut AstBuilder, for_stmt: &ForStatement) -> Entity {
        let span = self.position_to_span(&for_stmt.pos);
        let body = self.convert_block_to_block_expression(builder, &for_stmt.body);
        // Note: ForStatement structure needs pattern, iterator, etc. - simplified for now
        builder.create_for_statement(&span, body)
    }

    fn convert_struct(&self, builder: &mut AstBuilder, struct_stmt: &StructStatement) -> Entity {
        let _span = self.position_to_span(&struct_stmt.pos);
        builder.create_root(vec![])
    }

    fn convert_enum(&self, builder: &mut AstBuilder, enum_stmt: &EnumStatement) -> Entity {
        let _span = self.position_to_span(&enum_stmt.pos);
        builder.create_root(vec![])
    }

    fn convert_interface(&self, builder: &mut AstBuilder, iface: &InterfaceStatement) -> Entity {
        let _span = self.position_to_span(&iface.pos);
        builder.create_root(vec![])
    }

    fn convert_impl(&self, builder: &mut AstBuilder, impl_stmt: &ImplStatement) -> Entity {
        let _span = self.position_to_span(&impl_stmt.pos);
        builder.create_root(vec![])
    }

    fn convert_expression_statement(&self, builder: &mut AstBuilder, expr: &ExpressionStatement) -> Entity {
        self.convert_expression(builder, &expr.expression)
    }

    fn convert_expression(&self, builder: &mut AstBuilder, expr: &ExpressionKind) -> Entity {
        match expr {
            ExpressionKind::If(if_expr) => self.convert_if_expression(builder, if_expr),
            ExpressionKind::Case(case_expr) => self.convert_case_expression(builder, case_expr),
            ExpressionKind::Lambda(lambda) => self.convert_lambda(builder, lambda),
            ExpressionKind::Block(block) => self.convert_block_expression(builder, block),
            ExpressionKind::Type(type_expr) => self.convert_type_expression(builder, type_expr),
            ExpressionKind::Operation(op) => self.convert_operation(builder, op),
        }
    }

    fn convert_if_expression(&self, builder: &mut AstBuilder, if_expr: &IfExpression) -> Entity {
        let span = self.position_to_span(&if_expr.pos);
        let mut branches = Vec::new();
        
        if let Some(if_branch) = &if_expr.if_branch {
            branches.push(self.convert_if_branch(builder, if_branch));
        }
        
        for else_if in &if_expr.else_ifs {
            branches.push(self.convert_if_branch(builder, else_if));
        }
        
        if let Some(else_expr) = &if_expr.else_branch {
            let else_span = self.position_to_span(else_expr.position());
            let consequent = self.convert_expression(builder, else_expr);
            branches.push(builder.create_if_branch(&else_span, None, consequent));
        }
        
        builder.create_if_expression(&span, true, branches)
    }

    fn convert_if_branch(&self, builder: &mut AstBuilder, branch: &IfBranch) -> Entity {
        let span = self.position_to_span(&branch.pos);
        let condition = Some(self.convert_expression(builder, &branch.condition));
        let consequent = self.convert_expression(builder, &branch.then);
        builder.create_if_branch(&span, condition, consequent)
    }

    fn convert_case_expression(&self, builder: &mut AstBuilder, case: &CaseExpression) -> Entity {
        let _span = self.position_to_span(&case.pos);
        builder.create_root(vec![])
    }

    fn convert_lambda(&self, builder: &mut AstBuilder, lambda: &Lambda) -> Entity {
        let span = self.position_to_span(&lambda.pos);
        let parameters: Vec<Entity> = lambda.parameters.iter()
            .map(|param| self.convert_function_parameter(builder, param))
            .collect();
        let return_type = lambda.return_type.as_ref()
            .map(|atom| self.convert_atom(builder, atom));
        let body = self.convert_block_to_block_expression(builder, &lambda.body);
        builder.create_lambda(&span, parameters, return_type, body)
    }

    fn convert_block_expression(&self, builder: &mut AstBuilder, block: &BlockExpression) -> Entity {
        let span = self.position_to_span(&block.pos);
        let statements: Vec<Entity> = block.block.statements.iter()
            .map(|stmt| self.convert_decorated_statement(builder, stmt))
            .collect();
        builder.create_block_expression(&span, block.block.label.clone(), block.is_static, statements)
    }

    fn convert_type_expression(&self, builder: &mut AstBuilder, type_expr: &TypeExpression) -> Entity {
        let span = self.position_to_span(&type_expr.pos);
        let type_entity = self.convert_type(builder, &type_expr.type_kind);
        builder.create_type_expression(&span, type_entity)
    }

    fn convert_operation(&self, builder: &mut AstBuilder, op: &Operation) -> Entity {
        let span = self.position_to_span(&op.pos);
        let operator = op.prefix.first().cloned().unwrap_or_default();
        
        let operand_entities: Vec<Entity> = op.operands.iter()
            .map(|operand| self.convert_operand(builder, operand))
            .collect();
        
        let left = operand_entities.get(0).cloned();
        let right = operand_entities.get(1).cloned();
        
        builder.create_operation(&span, operator, left, right, None)
    }

    fn convert_operand(&self, builder: &mut AstBuilder, operand: &Operand) -> Entity {
        let base = self.convert_atom(builder, &operand.atom);
        
        if !operand.postfix.is_empty() {
            let ops: Vec<(Span<'_>, PostfixOpKind)> = operand.postfix.iter()
                .map(|postfix| (self.position_to_span(&postfix.pos), self.convert_postfix(builder, postfix)))
                .collect();
            builder.create_postfix_op(base, ops)
        } else {
            base
        }
    }

    fn convert_postfix(&self, builder: &mut AstBuilder, postfix: &Postfix) -> PostfixOpKind {
        if !postfix.invocation.is_empty() {
            let arg_entities: Vec<Entity> = postfix.invocation.iter()
                .map(|arg| self.convert_expression(builder, arg))
                .collect();
            PostfixOpKind::Invocation(arg_entities)
        } else if let Some(index_expr) = &postfix.index {
            let index_entity = self.convert_expression(builder, index_expr);
            PostfixOpKind::Index(index_entity)
        } else if let Some(operator) = &postfix.operator {
            PostfixOpKind::Operator(operator.clone())
        } else {
            PostfixOpKind::Invocation(vec![])
        }
    }

    fn convert_atom(&self, builder: &mut AstBuilder, atom: &AtomKind) -> Entity {
        match atom {
            AtomKind::Grouped(grouped) => self.convert_grouped(builder, grouped),
            AtomKind::Type(type_atom) => self.convert_type_atom(builder, type_atom),
            AtomKind::Value(value_atom) => self.convert_value_atom(builder, value_atom),
            AtomKind::Binding(binding) => self.convert_binding(builder, binding),
        }
    }

    fn convert_grouped(&self, builder: &mut AstBuilder, grouped: &GroupedExpression) -> Entity {
        let span = self.position_to_span(&grouped.pos);
        if let Some(expression) = &grouped.expression {
            let expr_entity = self.convert_expression(builder, expression);
            builder.create_grouped_expression(&span, expr_entity)
        } else {
            builder.create_root(vec![])
        }
    }

    fn convert_type_atom(&self, builder: &mut AstBuilder, type_atom: &AtomicType) -> Entity {
        let span = self.position_to_span(&type_atom.pos);
        if let Some(type_expr) = &type_atom.type_expr {
            let type_entity = self.convert_type(builder, type_expr);
            builder.create_type_atom(&span, type_entity)
        } else {
            builder.create_root(vec![])
        }
    }

    fn convert_value_atom(&self, builder: &mut AstBuilder, value_atom: &AtomicValue) -> Entity {
        let span = self.position_to_span(&value_atom.pos);
        if let Some(value) = &value_atom.value {
            let value_entity = self.convert_value(builder, value);
            builder.create_value_atom(&span, value_entity)
        } else {
            builder.create_root(vec![])
        }
    }

    fn convert_binding(&self, builder: &mut AstBuilder, binding: &Binding) -> Entity {
        let span = self.position_to_span(&binding.pos);
        builder.create_binding(&span, vec![binding.name.clone()])
    }

    fn convert_type(&self, builder: &mut AstBuilder, type_kind: &TypeKind) -> Entity {
        match type_kind {
            TypeKind::Pointer(ptr) => self.convert_pointer_type(builder, ptr),
            TypeKind::Array(arr) => self.convert_array_type(builder, arr),
            TypeKind::Integer(int) => self.convert_integer_type(builder, int),
            TypeKind::Floating(float) => self.convert_floating_type(builder, float),
            TypeKind::Void(void) => self.convert_void_type(builder, void),
            TypeKind::Self_(self_type) => self.convert_self_type(builder, self_type),
            TypeKind::Function(func) => self.convert_function_type(builder, func),
        }
    }

    fn convert_pointer_type(&self, builder: &mut AstBuilder, ptr: &PointerType) -> Entity {
        let span = self.position_to_span(&ptr.pos);
        let target = self.convert_atom(builder, &ptr.target);
        builder.create_pointer_type(&span, target, ptr.is_owned, ptr.is_raw, ptr.is_mutable)
    }

    fn convert_array_type(&self, builder: &mut AstBuilder, arr: &ArrayType) -> Entity {
        let span = self.position_to_span(&arr.pos);
        let element_type = self.convert_atom(builder, &arr.element);
        builder.create_array_type(&span, element_type, arr.dimension)
    }

    fn convert_integer_type(&self, builder: &mut AstBuilder, int: &IntegerType) -> Entity {
        let span = self.position_to_span(&int.pos);
        builder.create_integer_type(&span, int.signed, Some(int.bit_size))
    }

    fn convert_floating_type(&self, builder: &mut AstBuilder, float: &FloatingType) -> Entity {
        let _span = self.position_to_span(&float.pos);
        builder.create_root(vec![])
    }

    fn convert_void_type(&self, builder: &mut AstBuilder, void: &VoidType) -> Entity {
        let _span = self.position_to_span(&void.pos);
        builder.create_root(vec![])
    }

    fn convert_self_type(&self, builder: &mut AstBuilder, self_type: &SelfType) -> Entity {
        let _span = self.position_to_span(&self_type.pos);
        builder.create_root(vec![])
    }

    fn convert_function_type(&self, builder: &mut AstBuilder, func: &FunctionType) -> Entity {
        let span = self.position_to_span(&func.pos);
        let parameters: Vec<Entity> = func.parameters.iter()
            .map(|param| self.convert_function_parameter(builder, param))
            .collect();
        let return_type = func.return_type.as_ref()
            .map(|atom| self.convert_atom(builder, atom));
        builder.create_function_type(&span, func.name.clone(), parameters, return_type)
    }

    fn convert_value(&self, builder: &mut AstBuilder, value: &ValueKind) -> Entity {
        match value {
            ValueKind::Integer(int) => self.convert_integer_value(builder, int),
            ValueKind::Boolean(bool) => self.convert_boolean_value(builder, bool),
            ValueKind::Float(float) => self.convert_float_value(builder, float),
            ValueKind::Null(null) => self.convert_null_value(builder, null),
            ValueKind::String(string) => self.convert_string_value(builder, string),
        }
    }

    fn convert_integer_value(&self, builder: &mut AstBuilder, int: &Integer) -> Entity {
        let span = self.position_to_span(&int.pos);
        builder.create_integer_value(&span, int.value)
    }

    fn convert_boolean_value(&self, builder: &mut AstBuilder, bool: &Boolean) -> Entity {
        let span = self.position_to_span(&bool.pos);
        builder.create_boolean_value(&span, bool.value)
    }

    fn convert_float_value(&self, builder: &mut AstBuilder, float: &Float) -> Entity {
        let _span = self.position_to_span(&float.pos);
        builder.create_root(vec![])
    }

    fn convert_null_value(&self, builder: &mut AstBuilder, null: &Null) -> Entity {
        let _span = self.position_to_span(&null.pos);
        builder.create_root(vec![])
    }

    fn convert_string_value(&self, builder: &mut AstBuilder, string: &StringLiteral) -> Entity {
        let span = self.position_to_span(&string.pos);
        let fragments: Vec<Entity> = string.fragments.iter()
            .map(|frag| self.convert_string_fragment(builder, frag))
            .collect();
        builder.create_string_value(&span, fragments)
    }

    fn convert_string_fragment(&self, builder: &mut AstBuilder, fragment: &StringFragment) -> Entity {
        let span = self.position_to_span(&fragment.pos);
        match &fragment.interpolation {
            Some(expr) => {
                let expr_entity = self.convert_expression(builder, expr);
                builder.create_string_interpolation(&span, expr_entity)
            }
            None => {
                let text = fragment.text.clone().unwrap_or_default();
                builder.create_string_fragment(&span, text)
            }
        }
    }

    fn position_to_span(&self, _pos: &Position) -> Span {
        // Create a dummy span from position information
        // Using a single space as source to ensure valid span
        Span::new(" ", 0, 1).unwrap()
    }
}