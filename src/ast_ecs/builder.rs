use hecs::{Entity, World};
use pest::Span;
use super::components::*;
use super::AstWorld;

pub struct AstBuilder<'a> {
    world: &'a mut World,
}

impl<'a> AstBuilder<'a> {
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }

    pub fn from_ast_world(ast_world: &'a mut AstWorld) -> Self {
        Self::new(&mut ast_world.world)
    }

    pub fn create_root(&mut self, children: Vec<Entity>) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::Root },
            Children { entities: children },
        ))
    }

    pub fn create_module(
        &mut self,
        span: &Span,
        name: String,
        declarations: Vec<Entity>,
    ) -> Entity {
        let entity = self.world.spawn((
            NodeType { kind: NodeKind::Module },
            PositionComponent::from_span(span),
            Name { value: name },
            DeclarationKind { entities: declarations.clone() },
        ));

        if !declarations.is_empty() {
            self.world.insert_one(entity, Children { entities: declarations }).unwrap();
        }

        entity
    }

    pub fn create_function(
        &mut self,
        span: &Span,
        name: String,
        is_static: bool,
        parameters: Vec<Entity>,
        return_type: Option<Entity>,
        body: Option<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Function }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, Name { value: name }).unwrap();

        // Insert optional components
        if is_static {
            self.world.insert_one(entity, StaticModifier { is_static: true }).unwrap();
        }

        if !parameters.is_empty() {
            self.world.insert_one(entity, Parameters { entities: parameters }).unwrap();
        }

        if let Some(ret_type) = return_type {
            self.world.insert_one(entity, ReturnTypeReference { entity: Some(ret_type) }).unwrap();
        }

        if let Some(body_entity) = body {
            self.world.insert_one(entity, BodyReference { entity: body_entity }).unwrap();
        }

        entity
    }

    pub fn create_let_statement(
        &mut self,
        span: &Span,
        is_static: bool,
        is_mutable: bool,
        binding: Entity,
        type_ref: Option<Entity>,
        value: Option<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Let }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, TargetReference { entity: binding }).unwrap();

        // Insert optional components
        if is_static {
            self.world.insert_one(entity, StaticModifier { is_static: true }).unwrap();
        }

        if is_mutable {
            self.world.insert_one(entity, MutableModifier { is_mutable: true }).unwrap();
        }

        if let Some(type_entity) = type_ref {
            self.world.insert_one(entity, TypeReference { type_entity: Some(type_entity) }).unwrap();
        }

        if let Some(value_entity) = value {
            self.world.insert_one(entity, ValueReference { value_entity: Some(value_entity) }).unwrap();
        }

        entity
    }

    pub fn create_return_statement(
        &mut self,
        span: &Span,
        tag: Option<String>,
        value: Option<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Return }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();

        // Insert optional components
        if let Some(tag_value) = tag {
            self.world.insert_one(entity, Tag { value: tag_value }).unwrap();
        }

        if let Some(value_entity) = value {
            self.world.insert_one(entity, ValueReference { value_entity: Some(value_entity) }).unwrap();
        }

        entity
    }

    pub fn create_break_statement(
        &mut self,
        span: &Span,
        label: Option<String>,
        value: Option<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Break }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();

        // Insert optional components
        if let Some(label_value) = label {
            self.world.insert_one(entity, Label { value: label_value }).unwrap();
        }

        if let Some(value_entity) = value {
            self.world.insert_one(entity, ValueReference { value_entity: Some(value_entity) }).unwrap();
        }

        entity
    }

    pub fn create_while_statement(
        &mut self,
        span: &Span,
        condition: Entity,
        body: Entity,
    ) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::While },
            PositionComponent::from_span(span),
            ConditionReference { entity: condition },
            BodyReference { entity: body },
        ))
    }

    pub fn create_for_statement(
        &mut self,
        span: &Span,
        body: Entity,
    ) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::For },
            PositionComponent::from_span(span),
            BodyReference { entity: body },
        ))
    }

    pub fn create_if_expression(
        &mut self,
        span: &Span,
        is_expression: bool,
        branches: Vec<Entity>,
    ) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::If },
            PositionComponent::from_span(span),
            IsExpression { value: is_expression },
            Branches { entities: branches },
        ))
    }

    pub fn create_if_branch(
        &mut self,
        span: &Span,
        condition: Option<Entity>,
        consequent: Entity,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::IfBranch }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, ConsequentReference { entity: consequent }).unwrap();

        // Insert optional components
        if let Some(cond) = condition {
            self.world.insert_one(entity, ConditionReference { entity: cond }).unwrap();
        }

        entity
    }

    pub fn create_block_expression(
        &mut self,
        span: &Span,
        label: Option<String>,
        is_static: bool,
        statements: Vec<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::BlockExpression }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, Statements { entities: statements }).unwrap();

        // Insert optional components
        if let Some(label_value) = label {
            self.world.insert_one(entity, Label { value: label_value }).unwrap();
        }

        if is_static {
            self.world.insert_one(entity, StaticModifier { is_static: true }).unwrap();
        }

        entity
    }

    pub fn create_integer_value(&mut self, span: &Span, value: u64) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::IntegerValue },
            PositionComponent::from_span(span),
            IntegerLiteral { value },
        ))
    }

    pub fn create_boolean_value(&mut self, span: &Span, value: bool) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::BooleanValue },
            PositionComponent::from_span(span),
            BooleanLiteral { value },
        ))
    }

    pub fn create_string_value(&mut self, span: &Span, fragments: Vec<Entity>) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::StringValue },
            PositionComponent::from_span(span),
            StringFragments { entities: fragments },
        ))
    }

    pub fn create_string_fragment(&mut self, span: &Span, content: String) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::StringFragment },
            PositionComponent::from_span(span),
            StringContent { value: content },
        ))
    }

    pub fn create_string_interpolation(&mut self, span: &Span, expression: Entity) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::StringFragment },
            PositionComponent::from_span(span),
            StringInterpolation { expression },
        ))
    }

    pub fn create_binding(&mut self, span: &Span, path: Vec<String>) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::Binding },
            PositionComponent::from_span(span),
            PathSegments { segments: path },
        ))
    }

    pub fn create_operation(
        &mut self,
        span: &Span,
        operator: String,
        left: Option<Entity>,
        right: Option<Entity>,
        operand: Option<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Operation }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, OperatorKind { operator }).unwrap();

        // Insert optional components
        if let Some(left_entity) = left {
            self.world.insert_one(entity, LeftOperand { entity: left_entity }).unwrap();
        }

        if let Some(right_entity) = right {
            self.world.insert_one(entity, RightOperand { entity: right_entity }).unwrap();
        }

        if let Some(operand_entity) = operand {
            self.world.insert_one(entity, OperandReference { entity: operand_entity }).unwrap();
        }

        entity
    }

    pub fn create_integer_type(
        &mut self,
        span: &Span,
        is_signed: bool,
        bits: Option<u64>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Integer }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, Signed { is_signed }).unwrap();

        // Insert optional components
        if let Some(bit_size) = bits {
            self.world.insert_one(entity, BitSize { size: bit_size }).unwrap();
        }

        entity
    }

    pub fn create_pointer_type(
        &mut self,
        span: &Span,
        target: Entity,
        is_owned: bool,
        is_raw: bool,
        is_mutable: bool,
    ) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::Pointer },
            PositionComponent::from_span(span),
            TargetReference { entity: target },
            PointerModifiers { is_owned, is_raw, is_mutable },
        ))
    }

    pub fn create_array_type(
        &mut self,
        span: &Span,
        element_type: Entity,
        dimension: Option<u64>,
    ) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::Array },
            PositionComponent::from_span(span),
            ElementTypeReference { entity: element_type },
            ArrayDimension { size: dimension },
        ))
    }

    pub fn create_function_type(
        &mut self,
        span: &Span,
        name: Option<String>,
        parameters: Vec<Entity>,
        return_type: Option<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::FunctionType }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();

        // Insert optional components
        if let Some(type_name) = name {
            self.world.insert_one(entity, Name { value: type_name }).unwrap();
        }

        if !parameters.is_empty() {
            self.world.insert_one(entity, Parameters { entities: parameters }).unwrap();
        }

        if let Some(ret_type) = return_type {
            self.world.insert_one(entity, ReturnTypeReference { entity: Some(ret_type) }).unwrap();
        }

        entity
    }

    pub fn create_function_parameter(
        &mut self,
        span: &Span,
        name: String,
        type_ref: Entity,
    ) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::FunctionParameter },
            PositionComponent::from_span(span),
            Name { value: name },
            TypeReference { type_entity: Some(type_ref) },
        ))
    }

    pub fn create_annotation(
        &mut self,
        span: &Span,
        name: String,
        arguments: Vec<Entity>,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Annotation }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, Name { value: name }).unwrap();

        // Insert optional components
        if !arguments.is_empty() {
            self.world.insert_one(entity, Arguments { entities: arguments }).unwrap();
        }

        entity
    }

    pub fn create_decorated_statement(
        &mut self,
        span: &Span,
        comments: Vec<Entity>,
        annotations: Vec<Entity>,
        statement: Entity,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::DecoratedStatement }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, TargetReference { entity: statement }).unwrap();

        // Insert optional components
        if !comments.is_empty() {
            self.world.insert_one(entity, Comments { entities: comments }).unwrap();
        }

        if !annotations.is_empty() {
            self.world.insert_one(entity, Annotations { entities: annotations }).unwrap();
        }

        entity
    }

    pub fn create_comment(&mut self, span: &Span, text: String, is_doc: bool) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::Comment },
            PositionComponent::from_span(span),
            CommentText { text, is_doc },
        ))
    }

    pub fn create_type_atom(&mut self, span: &Span, type_entity: Entity) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::TypeAtom },
            PositionComponent::from_span(span),
            TypeReference { type_entity: Some(type_entity) },
        ))
    }

    pub fn create_value_atom(&mut self, span: &Span, value_entity: Entity) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::ValueAtom },
            PositionComponent::from_span(span),
            ValueReference { value_entity: Some(value_entity) },
        ))
    }

    pub fn create_grouped_expression(&mut self, span: &Span, expression: Entity) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::Grouped },
            PositionComponent::from_span(span),
            TargetReference { entity: expression },
        ))
    }

    pub fn create_lambda(
        &mut self,
        span: &Span,
        parameters: Vec<Entity>,
        return_type: Option<Entity>,
        body: Entity,
    ) -> Entity {
        let entity = self.world.reserve_entity();
        
        // Insert required components
        self.world.insert_one(entity, NodeType { kind: NodeKind::Lambda }).unwrap();
        self.world.insert_one(entity, PositionComponent::from_span(span)).unwrap();
        self.world.insert_one(entity, BodyReference { entity: body }).unwrap();

        // Insert optional components
        if !parameters.is_empty() {
            self.world.insert_one(entity, Parameters { entities: parameters }).unwrap();
        }

        if let Some(ret_type) = return_type {
            self.world.insert_one(entity, ReturnTypeReference { entity: Some(ret_type) }).unwrap();
        }

        entity
    }

    pub fn create_type_expression(&mut self, span: &Span, type_entity: Entity) -> Entity {
        self.world.spawn((
            NodeType { kind: NodeKind::TypeExpression },
            PositionComponent::from_span(span),
            TypeReference { type_entity: Some(type_entity) },
        ))
    }

    pub fn create_postfix_op(
        &mut self,
        base: Entity,
        operations: Vec<(Span, PostfixOpKind)>,
    ) -> Entity {
        let mut current = base;
        
        for (span, op_kind) in operations {
            let operation = PostfixOperation { kind: op_kind };
            let new_entity = self.world.spawn((
                NodeType { kind: NodeKind::PostfixOp },
                PositionComponent::from_span(&span),
                TargetReference { entity: current },
                operation,
            ));
            current = new_entity;
        }
        
        current
    }

    pub fn set_parent(&mut self, child: Entity, parent: Entity) {
        self.world.insert_one(child, Parent { entity: parent }).unwrap();
    }

    pub fn add_child(&mut self, parent: Entity, child: Entity) {
        // Try to get existing children component
        let has_children = self.world.get::<&Children>(parent).is_ok();
        
        if has_children {
            // Get mutable access to modify existing children
            if let Ok(mut children) = self.world.get::<&mut Children>(parent) {
                children.add(child);
            }
        } else {
            // Insert new children component
            self.world.insert_one(parent, Children { entities: vec![child] }).unwrap();
        }
    }
}