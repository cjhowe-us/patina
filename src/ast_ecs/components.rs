use hecs::Entity;
use pest::Span;
use crate::ast::Position;

#[derive(Debug, Clone, PartialEq)]
pub struct PositionComponent {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl PositionComponent {
    pub fn from_span(span: &Span) -> Self {
        let (line, column) = span.start_pos().line_col();
        Self {
            line,
            column,
            length: span.end() - span.start(),
        }
    }

    pub fn to_position(&self) -> Position {
        Position {
            filename: None,
            line: self.line,
            column: self.column,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeType {
    pub kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    // Structural
    Root,
    Block,
    DecoratedStatement,
    Comment,
    Annotation,
    
    // Statements
    Module,
    Function,
    Return,
    Let,
    Assignment,
    Break,
    While,
    For,
    Struct,
    Enum,
    Interface,
    Impl,
    ExpressionStatement,
    
    // Expressions
    If,
    Case,
    Lambda,
    BlockExpression,
    TypeExpression,
    Operation,
    
    // Atoms
    Grouped,
    TypeAtom,
    ValueAtom,
    Binding,
    
    // Types
    Pointer,
    Array,
    Integer,
    Floating,
    Void,
    SelfType,
    FunctionType,
    
    // Values
    IntegerValue,
    BooleanValue,
    FloatValue,
    NullValue,
    StringValue,
    
    // Supporting nodes
    FunctionParameter,
    IfBranch,
    CaseBranch,
    StringFragment,
    PostfixOp,
    StructField,
    EnumVariant,
    InterfaceMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeReference {
    pub type_entity: Option<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueReference {
    pub value_entity: Option<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticModifier {
    pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutableModifier {
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerModifiers {
    pub is_owned: bool,
    pub is_raw: bool,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Children {
    pub entities: Vec<Entity>,
}

impl Children {
    pub fn new() -> Self {
        Self { entities: Vec::new() }
    }

    pub fn add(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parent {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameters {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statements {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comments {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotations {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParameters {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitSize {
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signed {
    pub is_signed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayDimension {
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringFragments {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringContent {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringInterpolation {
    pub expression: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerLiteral {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLiteral {
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanLiteral {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorKind {
    pub operator: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostfixOperation {
    pub kind: PostfixOpKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOpKind {
    Index(Entity),
    Invocation(Vec<Entity>),
    Operator(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsequentReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlternativeReference {
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementTypeReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnTypeReference {
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathSegments {
    pub segments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommentText {
    pub text: String,
    pub is_doc: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationKind {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fields {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variants {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Methods {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Implementations {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branches {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperandReference {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeftOperand {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RightOperand {
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsExpression {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBranch {
    pub entity: Option<Entity>,
}