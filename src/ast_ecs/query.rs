use hecs::{Entity, World};
use super::components::*;
use crate::ast::Position;

pub struct AstQuery<'a> {
    world: &'a World,
}

impl<'a> AstQuery<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    pub fn get_position(&self, entity: Entity) -> Option<Position> {
        self.world.get::<&PositionComponent>(entity)
            .ok()
            .map(|pos| pos.to_position())
    }

    pub fn get_node_kind(&self, entity: Entity) -> Option<NodeKind> {
        self.world.get::<&NodeType>(entity)
            .ok()
            .map(|node_type| node_type.kind.clone())
    }

    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        let mut children = Vec::new();
        
        // Check for Children component
        if let Ok(child_component) = self.world.get::<&Children>(entity) {
            children.extend(child_component.entities.clone());
        }
        
        // Check for TargetReference (used by DecoratedStatement)
        if let Ok(target_ref) = self.world.get::<&TargetReference>(entity) {
            children.push(target_ref.entity);
        }
        
        // Check for ValueReference
        if let Ok(value_ref) = self.world.get::<&ValueReference>(entity) {
            if let Some(value_entity) = value_ref.value_entity {
                children.push(value_entity);
            }
        }
        
        // Check for TypeReference
        if let Ok(type_ref) = self.world.get::<&TypeReference>(entity) {
            if let Some(type_entity) = type_ref.type_entity {
                children.push(type_entity);
            }
        }
        
        // Check for BodyReference
        if let Ok(body_ref) = self.world.get::<&BodyReference>(entity) {
            children.push(body_ref.entity);
        }
        
        // Check for Parameters
        if let Ok(params) = self.world.get::<&Parameters>(entity) {
            children.extend(params.entities.clone());
        }
        
        // Check for Branches (for if/case expressions)
        if let Ok(branches) = self.world.get::<&Branches>(entity) {
            children.extend(branches.entities.clone());
        }
        
        // Check for Comments
        if let Ok(comments) = self.world.get::<&Comments>(entity) {
            children.extend(comments.entities.clone());
        }
        
        // Check for Annotations
        if let Ok(annotations) = self.world.get::<&Annotations>(entity) {
            children.extend(annotations.entities.clone());
        }
        
        // Check for other reference components
        if let Ok(condition_ref) = self.world.get::<&ConditionReference>(entity) {
            children.push(condition_ref.entity);
        }
        
        if let Ok(consequent_ref) = self.world.get::<&ConsequentReference>(entity) {
            children.push(consequent_ref.entity);
        }
        
        if let Ok(return_type_ref) = self.world.get::<&ReturnTypeReference>(entity) {
            if let Some(return_type) = return_type_ref.entity {
                children.push(return_type);
            }
        }
        
        children
    }

    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&Parent>(entity)
            .ok()
            .map(|parent| parent.entity)
    }

    pub fn get_name(&self, entity: Entity) -> Option<String> {
        self.world.get::<&Name>(entity)
            .ok()
            .map(|name| name.value.clone())
    }

    pub fn get_label(&self, entity: Entity) -> Option<String> {
        self.world.get::<&Label>(entity)
            .ok()
            .map(|label| label.value.clone())
    }

    pub fn get_tag(&self, entity: Entity) -> Option<String> {
        self.world.get::<&Tag>(entity)
            .ok()
            .map(|tag| tag.value.clone())
    }

    pub fn is_static(&self, entity: Entity) -> bool {
        self.world.get::<&StaticModifier>(entity)
            .ok()
            .map(|s| s.is_static)
            .unwrap_or(false)
    }

    pub fn is_mutable(&self, entity: Entity) -> bool {
        self.world.get::<&MutableModifier>(entity)
            .ok()
            .map(|m| m.is_mutable)
            .unwrap_or(false)
    }

    pub fn get_type_reference(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&TypeReference>(entity)
            .ok()
            .and_then(|type_ref| type_ref.type_entity)
    }

    pub fn get_value_reference(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&ValueReference>(entity)
            .ok()
            .and_then(|value_ref| value_ref.value_entity)
    }

    pub fn get_parameters(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Parameters>(entity)
            .ok()
            .map(|params| params.entities.clone())
            .unwrap_or_default()
    }

    pub fn get_statements(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Statements>(entity)
            .ok()
            .map(|stmts| stmts.entities.clone())
            .unwrap_or_default()
    }

    pub fn walk_children<F>(&self, entity: Entity, visitor: &mut F)
    where
        F: FnMut(Entity),
    {
        visitor(entity);
        for child in self.get_children(entity) {
            self.walk_children(child, visitor);
        }
    }

    pub fn walk_children_with_depth<F>(&self, entity: Entity, depth: usize, visitor: &mut F)
    where
        F: FnMut(Entity, usize),
    {
        visitor(entity, depth);
        for child in self.get_children(entity) {
            self.walk_children_with_depth(child, depth + 1, visitor);
        }
    }

    pub fn find_by_position(&self, line: usize, column: usize) -> Vec<Entity> {
        let mut results = Vec::new();
        for (entity, pos) in self.world.query::<&PositionComponent>().iter() {
            if pos.line == line && pos.column <= column && column < pos.column + pos.length {
                results.push(entity);
            }
        }
        results
    }

    pub fn find_by_name(&self, name: &str) -> Vec<Entity> {
        let mut results = Vec::new();
        for (entity, name_comp) in self.world.query::<&Name>().iter() {
            if name_comp.value == name {
                results.push(entity);
            }
        }
        results
    }

    pub fn find_by_kind(&self, kind: NodeKind) -> Vec<Entity> {
        let mut results = Vec::new();
        for (entity, node_type) in self.world.query::<&NodeType>().iter() {
            if node_type.kind == kind {
                results.push(entity);
            }
        }
        results
    }

    pub fn get_all_functions(&self) -> Vec<Entity> {
        self.find_by_kind(NodeKind::Function)
    }

    pub fn get_all_modules(&self) -> Vec<Entity> {
        self.find_by_kind(NodeKind::Module)
    }

    pub fn get_integer_value(&self, entity: Entity) -> Option<u64> {
        self.world.get::<&IntegerLiteral>(entity)
            .ok()
            .map(|lit| lit.value)
    }

    pub fn get_boolean_value(&self, entity: Entity) -> Option<bool> {
        self.world.get::<&BooleanLiteral>(entity)
            .ok()
            .map(|lit| lit.value)
    }

    pub fn get_string_fragments(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&StringFragments>(entity)
            .ok()
            .map(|frags| frags.entities.clone())
            .unwrap_or_default()
    }

    pub fn get_string_content(&self, entity: Entity) -> Option<String> {
        self.world.get::<&StringContent>(entity)
            .ok()
            .map(|content| content.value.clone())
    }

    pub fn get_string_interpolation(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&StringInterpolation>(entity)
            .ok()
            .map(|interp| interp.expression)
    }

    pub fn get_pointer_modifiers(&self, entity: Entity) -> Option<PointerModifiers> {
        self.world.get::<&PointerModifiers>(entity)
            .ok()
            .map(|mods| (*mods).clone())
    }

    pub fn get_array_dimension(&self, entity: Entity) -> Option<Option<u64>> {
        self.world.get::<&ArrayDimension>(entity)
            .ok()
            .map(|dim| dim.size)
    }

    pub fn get_bit_size(&self, entity: Entity) -> Option<u64> {
        self.world.get::<&BitSize>(entity)
            .ok()
            .map(|size| size.size)
    }

    pub fn is_signed(&self, entity: Entity) -> Option<bool> {
        self.world.get::<&Signed>(entity)
            .ok()
            .map(|signed| signed.is_signed)
    }

    pub fn get_path_segments(&self, entity: Entity) -> Option<Vec<String>> {
        self.world.get::<&PathSegments>(entity)
            .ok()
            .map(|path| path.segments.clone())
    }

    pub fn get_operator(&self, entity: Entity) -> Option<String> {
        self.world.get::<&OperatorKind>(entity)
            .ok()
            .map(|op| op.operator.clone())
    }

    pub fn get_left_operand(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&LeftOperand>(entity)
            .ok()
            .map(|left| left.entity)
    }

    pub fn get_right_operand(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&RightOperand>(entity)
            .ok()
            .map(|right| right.entity)
    }

    pub fn get_body(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&BodyReference>(entity)
            .ok()
            .map(|body| body.entity)
    }

    pub fn get_condition(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&ConditionReference>(entity)
            .ok()
            .map(|cond| cond.entity)
    }

    pub fn get_consequent(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&ConsequentReference>(entity)
            .ok()
            .map(|cons| cons.entity)
    }

    pub fn get_alternative(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&AlternativeReference>(entity)
            .ok()
            .and_then(|alt| alt.entity)
    }

    pub fn get_return_type(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&ReturnTypeReference>(entity)
            .ok()
            .and_then(|ret| ret.entity)
    }

    pub fn get_target(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&TargetReference>(entity)
            .ok()
            .map(|target| target.entity)
    }

    pub fn get_element_type(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&ElementTypeReference>(entity)
            .ok()
            .map(|elem| elem.entity)
    }

    pub fn get_branches(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Branches>(entity)
            .ok()
            .map(|branches| branches.entities.clone())
            .unwrap_or_default()
    }

    pub fn get_arguments(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Arguments>(entity)
            .ok()
            .map(|args| args.entities.clone())
            .unwrap_or_default()
    }

    pub fn get_annotations(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Annotations>(entity)
            .ok()
            .map(|annots| annots.entities.clone())
            .unwrap_or_default()
    }

    pub fn get_comments(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Comments>(entity)
            .ok()
            .map(|comments| comments.entities.clone())
            .unwrap_or_default()
    }

    pub fn get_comment_text(&self, entity: Entity) -> Option<(String, bool)> {
        self.world.get::<&CommentText>(entity)
            .ok()
            .map(|comment| (comment.text.clone(), comment.is_doc))
    }

    pub fn get_postfix_operation(&self, entity: Entity) -> Option<PostfixOpKind> {
        self.world.get::<&PostfixOperation>(entity)
            .ok()
            .map(|op| op.kind.clone())
    }

    pub fn is_expression(&self, entity: Entity) -> bool {
        self.world.get::<&IsExpression>(entity)
            .ok()
            .map(|is_expr| is_expr.value)
            .unwrap_or(false)
    }

    pub fn get_float_value(&self, entity: Entity) -> Option<f64> {
        self.world.get::<&FloatLiteral>(entity)
            .ok()
            .map(|lit| lit.value)
    }
    
    pub fn get_generic_parameters(&self, entity: Entity) -> Vec<String> {
        self.world.get::<&GenericParameters>(entity)
            .ok()
            .map(|params| params.names.clone())
            .unwrap_or_default()
    }
    
    pub fn get_fields(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Fields>(entity)
            .ok()
            .map(|fields| fields.entities.clone())
            .unwrap_or_default()
    }
    
    pub fn get_variants(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Variants>(entity)
            .ok()
            .map(|variants| variants.entities.clone())
            .unwrap_or_default()
    }
    
    pub fn get_methods(&self, entity: Entity) -> Vec<Entity> {
        self.world.get::<&Methods>(entity)
            .ok()
            .map(|methods| methods.entities.clone())
            .unwrap_or_default()
    }
    
    pub fn get_declarations(&self, entity: Entity) -> Vec<Entity> {
        // Declarations are stored as Children in modules
        self.get_children(entity)
    }
    
    pub fn get_operand(&self, entity: Entity) -> Option<Entity> {
        self.world.get::<&OperandReference>(entity)
            .ok()
            .map(|op| op.entity)
    }
}

pub fn walk_ast<F>(world: &World, root: Entity, visitor: &mut F)
where
    F: FnMut(Entity),
{
    let query = AstQuery::new(world);
    query.walk_children(root, visitor);
}

pub fn walk_ast_with_callbacks<F, G>(
    world: &World,
    root: Entity,
    pre_visit: &mut F,
    post_visit: &mut G,
)
where
    F: FnMut(Entity),
    G: FnMut(Entity),
{
    let query = AstQuery::new(world);
    walk_with_callbacks_helper(&query, root, pre_visit, post_visit);
}

fn walk_with_callbacks_helper<F, G>(
    query: &AstQuery,
    entity: Entity,
    pre_visit: &mut F,
    post_visit: &mut G,
)
where
    F: FnMut(Entity),
    G: FnMut(Entity),
{
    pre_visit(entity);
    for child in query.get_children(entity) {
        walk_with_callbacks_helper(query, child, pre_visit, post_visit);
    }
    post_visit(entity);
}

pub fn walk_ast_until<F>(world: &World, root: Entity, visitor: &mut F) -> bool
where
    F: FnMut(Entity) -> bool,
{
    let query = AstQuery::new(world);
    walk_until_helper(&query, root, visitor)
}

fn walk_until_helper<F>(query: &AstQuery, entity: Entity, visitor: &mut F) -> bool
where
    F: FnMut(Entity) -> bool,
{
    if !visitor(entity) {
        return false;
    }
    for child in query.get_children(entity) {
        if !walk_until_helper(query, child, visitor) {
            return false;
        }
    }
    true
}

pub fn walk_ast_map<F, T>(world: &World, root: Entity, visitor: &mut F) -> Vec<T>
where
    F: FnMut(Entity) -> Option<T>,
{
    let mut results = Vec::new();
    let query = AstQuery::new(world);
    walk_map_helper(&query, root, visitor, &mut results);
    results
}

fn walk_map_helper<F, T>(
    query: &AstQuery,
    entity: Entity,
    visitor: &mut F,
    results: &mut Vec<T>,
)
where
    F: FnMut(Entity) -> Option<T>,
{
    if let Some(result) = visitor(entity) {
        results.push(result);
    }
    for child in query.get_children(entity) {
        walk_map_helper(query, child, visitor, results);
    }
}