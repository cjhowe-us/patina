/// Parser implementation using Pest parser generator with trait-based parsing
use crate::ast::*;
use hecs::Entity;
use pest::iterators::{Pair, Pairs};
use pest::Parser as PestParser;
use pest_derive::Parser;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PatinaParser;

/// Parse error type
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Trait for parseable nodes
pub trait ParseNode: Sized {
    /// Parse this node type from a Pest pair
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self;
}

/// Parser context for tracking state during parsing
pub struct ParserContext {
    pub filename: Option<String>,
}

impl ParserContext {
    pub fn new(filename: Option<String>) -> Self {
        Self { filename }
    }
}

/// Parse a Patina source file
pub fn parse_file(input: &str) -> Result<Ast, Box<dyn std::error::Error>> {
    parse_file_with_name("", input)
}

/// Parse a Patina source file with a given filename
pub fn parse_file_with_name(
    filename: &str,
    input: &str,
) -> Result<Ast, Box<dyn std::error::Error>> {
    let pairs = PatinaParser::parse(Rule::root, input)?;
    let mut ast = Ast::new();
    
    // Update root entity with position
    ast.add_component(ast.root.entity, Position {
        filename: Some(filename.to_string()),
        line: 1,
        column: 1,
        offset: 0,
        length: input.len(),
    });
    
    // Use parser visitor to build AST
    let mut visitor = ParserVisitor::new(filename);
    visitor.parse_root(&mut ast, pairs);
    
    Ok(ast)
}

/// Visitor for parsing Pest pairs into AST nodes
pub struct ParserVisitor {
    filename: String,
}

impl ParserVisitor {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
    
    pub fn parse_root(&mut self, ast: &mut Ast, pairs: Pairs<Rule>) {
        for pair in pairs {
            if pair.as_rule() == Rule::root {
                self.parse_root_inner(ast, pair);
            }
        }
    }
    
    fn parse_root_inner(&mut self, ast: &mut Ast, pair: Pair<Rule>) {
        for inner_pair in pair.into_inner() {
            if let Rule::decorated_statement = inner_pair.as_rule() {
                if let Some(stmt) = self.parse_decorated_statement(ast, inner_pair) {
                    ast.root.statements.push(stmt);
                }
            }
        }
    }
    
    fn parse_decorated_statement(&mut self, ast: &mut Ast, pair: Pair<Rule>) -> Option<Statement> {
        let mut comments = Vec::new();
        let mut annotations = Vec::new();
        let mut statement = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::line_comment | Rule::block_comment => {
                    comments.push(inner_pair.as_str().to_string());
                }
                Rule::annotation => {
                    annotations.push(self.parse_annotation(ast, inner_pair));
                }
                Rule::statement => {
                    statement = Some(Statement::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        // TODO: Attach comments and annotations to the statement entity
        
        statement
    }
    
    fn parse_annotation(&mut self, ast: &mut Ast, pair: Pair<Rule>) -> (String, Vec<Expression>) {
        let mut name = String::new();
        let mut args = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::tag => {
                    name = inner_pair.as_str()[1..].to_string(); // Skip @
                }
                Rule::expression => {
                    args.push(Expression::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        (name, args)
    }
    
    pub fn add_position(&self, ast: &mut Ast, entity: Entity, pair: &Pair<Rule>) {
        let span = pair.as_span();
        let (line, column) = span.start_pos().line_col();
        
        ast.add_component(entity, Position {
            filename: Some(self.filename.clone()),
            line,
            column,
            offset: span.start(),
            length: span.end() - span.start(),
        });
    }
}

/// Helper function to add position to an entity
fn add_position(ast: &mut Ast, entity: Entity, pair: &Pair<Rule>) {
    let span = pair.as_span();
    let (line, column) = span.start_pos().line_col();
    
    ast.add_component(entity, Position {
        filename: None, // TODO: Thread filename through
        line,
        column,
        offset: span.start(),
        length: span.end() - span.start(),
    });
}

// Implement ParseNode for Statement
impl ParseNode for Statement {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let inner = pair.into_inner().next().unwrap();
        
        match inner.as_rule() {
            Rule::module_stmt => Statement::Module(Module::parse(ast, inner)),
            Rule::function_stmt => Statement::Function(Function::parse(ast, inner)),
            Rule::let_stmt => Statement::Let(Let::parse(ast, inner)),
            Rule::return_stmt => Statement::Return(Return::parse(ast, inner)),
            Rule::break_stmt => Statement::Break(Break::parse(ast, inner)),
            Rule::while_stmt => Statement::While(While::parse(ast, inner)),
            Rule::for_stmt => Statement::For(For::parse(ast, inner)),
            Rule::static_stmt => Statement::Static(Static::parse(ast, inner)),
            Rule::expression_stmt => {
                let expr = Expression::parse(ast, inner.into_inner().next().unwrap());
                Statement::Expression(expr)
            }
            _ => todo!("Unhandled statement type: {:?}", inner.as_rule()),
        }
    }
}

// Implement ParseNode for Module
impl ParseNode for Module {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut name = String::new();
        let mut statements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::ident => name = inner_pair.as_str().to_string(),
                Rule::block => {
                    statements = parse_block_statements(ast, inner_pair);
                }
                _ => {}
            }
        }
        
        Module {
            entity,
            name,
            statements,
        }
    }
}

// Implement ParseNode for Function
impl ParseNode for Function {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut is_static = false;
        let mut name = String::new();
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut body = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::static_kw => is_static = true,
                Rule::ident => name = inner_pair.as_str().to_string(),
                Rule::function_parameter => {
                    parameters.push(Parameter::parse(ast, inner_pair));
                }
                Rule::atom => {
                    return_type = Some(parse_atom_as_expression(ast, inner_pair));
                }
                Rule::block => {
                    body = Some(Block::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        Function {
            entity,
            name,
            parameters,
            return_type,
            body,
            is_static,
        }
    }
}

// Implement ParseNode for Let
impl ParseNode for Let {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut is_static = false;
        let mut is_mutable = false;
        let mut pattern = None;
        let mut type_expr = None;
        let mut value = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::static_kw => is_static = true,
                Rule::mut_kw => is_mutable = true,
                Rule::ident => {
                    let ident_entity = ast.create_entity();
                    add_position(ast, ident_entity, &inner_pair);
                    pattern = Some(Pattern::Identifier(Identifier {
                        entity: ident_entity,
                        name: inner_pair.as_str().trim().to_string(),
                    }));
                }
                Rule::atom => {
                    type_expr = Some(parse_atom_as_expression(ast, inner_pair));
                }
                Rule::expression => {
                    value = Some(Expression::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        Let {
            entity,
            pattern: pattern.unwrap(),
            type_expr,
            value,
            is_mutable,
            is_static,
        }
    }
}

// Implement ParseNode for Return
impl ParseNode for Return {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut label = None;
        let mut value = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::label => {
                    label = Some(inner_pair.as_str()[1..].to_string()); // Skip ':'
                }
                Rule::expression => {
                    value = Some(Expression::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        Return {
            entity,
            label,
            value,
        }
    }
}

// Implement ParseNode for Break
impl ParseNode for Break {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut label = None;
        let mut value = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::label => {
                    label = Some(inner_pair.as_str()[1..].to_string()); // Skip ':'
                }
                Rule::expression => {
                    value = Some(Expression::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        Break {
            entity,
            label,
            value,
        }
    }
}

// Implement ParseNode for While
impl ParseNode for While {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut condition = None;
        let mut body = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::expression => {
                    condition = Some(Expression::parse(ast, inner_pair));
                }
                Rule::block => {
                    body = Some(Block::parse(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        While {
            entity,
            label: None, // TODO: Support labeled while loops
            condition: condition.unwrap(),
            body: body.unwrap(),
        }
    }
}

// Implement ParseNode for For
impl ParseNode for For {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        // TODO: Implement for loop parsing (it's complex with multiple forms)
        For {
            entity,
            label: None,
            init: None,
            condition: None,
            update: None,
            body: Block {
                entity: ast.create_entity(),
                label: None,
                statements: vec![],
            },
        }
    }
}

// Implement ParseNode for Static
impl ParseNode for Static {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut statements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::block {
                statements = parse_block_statements(ast, inner_pair);
            }
        }
        
        Static {
            entity,
            statements,
        }
    }
}

// Implement ParseNode for Expression
impl ParseNode for Expression {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let mut inner_pairs = pair.into_inner();
        
        // Handle empty expression
        let inner = match inner_pairs.next() {
            Some(inner) => inner,
            None => {
                // Return a placeholder for empty expressions
                let entity = ast.create_entity();
                return Expression::Literal(Literal::Null(NullLiteral { entity }));
            }
        };
        
        match inner.as_rule() {
            Rule::if_expression => Expression::If(Box::new(If::parse(ast, inner))),
            Rule::case_expression => Expression::Case(Box::new(Case::parse(ast, inner))),
            Rule::lambda => Expression::Lambda(Box::new(Lambda::parse(ast, inner))),
            Rule::block_expression => {
                let block = Block::parse(ast, inner.into_inner().next().unwrap());
                Expression::Block(block)
            }
            Rule::type_expression => parse_type_expression(ast, inner),
            Rule::operation => Operation::parse(ast, inner).into_expression(),
            _ => todo!("Unhandled expression type: {:?}", inner.as_rule()),
        }
    }
}

// Implement ParseNode for If
impl ParseNode for If {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut branches = Vec::new();
        let mut else_branch = None;
        
        let mut pairs = pair.into_inner();
        
        // First if branch
        if let Some(if_branch_pair) = pairs.next() {
            if if_branch_pair.as_rule() == Rule::if_branch {
                branches.push(IfBranch::parse(ast, if_branch_pair));
            }
        }
        
        // Check for else
        for inner_pair in pairs {
            match inner_pair.as_rule() {
                Rule::if_expression => {
                    // else if
                    let else_if = If::parse(ast, inner_pair);
                    else_branch = Some(Box::new(Expression::If(Box::new(else_if))));
                }
                Rule::block => {
                    // else block
                    let block = Block::parse(ast, inner_pair);
                    else_branch = Some(Box::new(Expression::Block(block)));
                }
                _ => {}
            }
        }
        
        If {
            entity,
            branches,
            else_branch,
        }
    }
}

// Implement ParseNode for IfBranch
impl ParseNode for IfBranch {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut condition = None;
        let mut body = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::expression => {
                    condition = Some(Expression::parse(ast, inner_pair));
                }
                Rule::block => {
                    let block = Block::parse(ast, inner_pair);
                    body = Some(Expression::Block(block));
                }
                _ => {}
            }
        }
        
        IfBranch {
            entity,
            condition: condition.unwrap(),
            body: body.unwrap(),
        }
    }
}

// Implement ParseNode for Case
impl ParseNode for Case {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        // TODO: Implement case expression parsing
        Case {
            entity,
            value: Box::new(Expression::Literal(Literal::Null(NullLiteral {
                entity: ast.create_entity(),
            }))),
            branches: vec![],
        }
    }
}

// Implement ParseNode for Lambda
impl ParseNode for Lambda {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut body = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::lambda_parameter => {
                    let param_entity = ast.create_entity();
                    add_position(ast, param_entity, &inner_pair);
                    
                    let mut name = String::new();
                    let mut type_expr = None;
                    
                    for param_inner in inner_pair.into_inner() {
                        match param_inner.as_rule() {
                            Rule::ident => name = param_inner.as_str().to_string(),
                            Rule::atom => {
                                type_expr = Some(parse_atom_as_expression(ast, param_inner));
                            }
                            _ => {}
                        }
                    }
                    
                    parameters.push(Parameter {
                        entity: param_entity,
                        name,
                        type_expr: type_expr.unwrap_or_else(|| {
                            // Default to placeholder type if not specified
                            Expression::Identifier(Identifier {
                                entity: ast.create_entity(),
                                name: "_".to_string(),
                            })
                        }),
                    });
                }
                Rule::atom => {
                    return_type = Some(Box::new(parse_atom_as_expression(ast, inner_pair)));
                }
                Rule::block => {
                    body = Some(Box::new(Expression::Block(Block::parse(ast, inner_pair))));
                }
                _ => {}
            }
        }
        
        Lambda {
            entity,
            parameters,
            return_type,
            body: body.unwrap(),
        }
    }
}

// Implement ParseNode for Operation
impl ParseNode for Operation {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut elements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::unary_operator => {
                    elements.push(OperationElement::Operator(inner_pair.as_str().to_string()));
                }
                Rule::operand => {
                    let operand = parse_operand(ast, inner_pair);
                    elements.push(OperationElement::Operand(operand));
                }
                Rule::operator => {
                    elements.push(OperationElement::Operator(inner_pair.as_str().to_string()));
                }
                _ => {}
            }
        }
        
        Operation { entity, elements }
    }
}

impl Operation {
    /// Convert operation to expression, handling single operand case
    fn into_expression(mut self) -> Expression {
        // If only one operand and no operators, return the operand directly
        if self.elements.len() == 1 {
            if let Some(OperationElement::Operand(expr)) = self.elements.pop() {
                return expr;
            }
        }
        
        Expression::Operation(self)
    }
}

// Implement ParseNode for Parameter
impl ParseNode for Parameter {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut name = String::new();
        let mut type_expr = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::ident => name = inner_pair.as_str().to_string(),
                Rule::atom => {
                    type_expr = Some(parse_atom_as_expression(ast, inner_pair));
                }
                _ => {}
            }
        }
        
        Parameter {
            entity,
            name,
            type_expr: type_expr.unwrap(),
        }
    }
}

// Implement ParseNode for Block
impl ParseNode for Block {
    fn parse(ast: &mut Ast, pair: Pair<Rule>) -> Self {
        let entity = ast.create_entity();
        add_position(ast, entity, &pair);
        
        let mut label = None;
        let mut statements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::label => {
                    label = Some(inner_pair.as_str()[1..].to_string());
                }
                Rule::decorated_statement => {
                    let mut visitor = ParserVisitor::new("");
                    if let Some(stmt) = visitor.parse_decorated_statement(ast, inner_pair) {
                        statements.push(stmt);
                    }
                }
                _ => {}
            }
        }
        
        Block {
            entity,
            label,
            statements,
        }
    }
}

// Helper functions

fn parse_operand(ast: &mut Ast, pair: Pair<Rule>) -> Expression {
    let mut expr = None;
    let mut postfixes = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::atom => {
                expr = Some(parse_atom_as_expression(ast, inner_pair));
            }
            Rule::postfix => {
                postfixes.push(inner_pair);
            }
            _ => {}
        }
    }
    
    let mut result = expr.unwrap();
    
    // Apply postfix operations
    for postfix in postfixes {
        result = apply_postfix(ast, result, postfix);
    }
    
    result
}

fn apply_postfix(ast: &mut Ast, expr: Expression, postfix: Pair<Rule>) -> Expression {
    // The postfix rule contains the actual operation (parentheses, brackets, or field)
    // The postfix can be one of three forms:
    // 1. Function call: (expr, expr, ...)
    // 2. Index: [expr]
    // 3. Field: .ident
    
    // Check what kind of postfix by looking at the first inner element
    let mut inner_pairs = postfix.into_inner();
    let first = inner_pairs.next();
    
    if let Some(first_pair) = first {
        match first_pair.as_rule() {
            Rule::left_paren => {
                // This is a function call - collect the arguments
                let entity = ast.create_entity();
                let mut arguments = Vec::new();
                
                // Collect expressions (skipping commas and right paren)
                for pair in inner_pairs {
                    if pair.as_rule() == Rule::expression {
                        arguments.push(Expression::parse(ast, pair));
                    }
                }
                
                Expression::Call(Call {
                    entity,
                    function: Box::new(expr),
                    arguments,
                })
            }
            Rule::left_bracket => {
                // This is an index operation - get the index expression
                let entity = ast.create_entity();
                
                // The next element should be the expression
                if let Some(expr_pair) = inner_pairs.next() {
                    if expr_pair.as_rule() == Rule::expression {
                        let index_expr = Expression::parse(ast, expr_pair);
                        Expression::Index(Index {
                            entity,
                            object: Box::new(expr),
                            index: Box::new(index_expr),
                        })
                    } else {
                        expr
                    }
                } else {
                    expr
                }
            }
            Rule::field => {
                // This is a field access - get the identifier  
                let entity = ast.create_entity();
                
                // The field rule should contain an ident
                if let Some(ident_pair) = first_pair.into_inner().next() {
                    if ident_pair.as_rule() == Rule::ident {
                        let field_name = ident_pair.as_str().to_string();
                        Expression::Field(Field {
                            entity,
                            object: Box::new(expr),
                            field: field_name,
                        })
                    } else {
                        expr
                    }
                } else {
                    expr
                }
            }
            _ => expr
        }
    } else {
        expr
    }
}

fn parse_atom_as_expression(ast: &mut Ast, pair: Pair<Rule>) -> Expression {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::grouped_expression => {
            Expression::parse(ast, inner.into_inner().next().unwrap())
        }
        Rule::atomic_value => parse_atomic_value(ast, inner),
        Rule::binding => {
            let entity = ast.create_entity();
            add_position(ast, entity, &inner);
            let ident = inner.into_inner().next().unwrap();
            let name = ident.as_str().to_string();
            Expression::Identifier(Identifier { entity, name })
        }
        Rule::atomic_type => {
            // Types are expressions in Patina
            parse_type_expression(ast, inner)
        }
        _ => todo!("Unhandled atom type: {:?}", inner.as_rule())
    }
}

fn parse_atomic_value(ast: &mut Ast, pair: Pair<Rule>) -> Expression {
    let inner = pair.into_inner().next()
        .expect("Atomic value should have inner content");
    
    match inner.as_rule() {
        Rule::integer => {
            let entity = ast.create_entity();
            add_position(ast, entity, &inner);
            let value = inner.as_str().trim().parse().unwrap();
            Expression::Literal(Literal::Integer(IntegerLiteral { entity, value }))
        }
        Rule::float => {
            let entity = ast.create_entity();
            add_position(ast, entity, &inner);
            let value = inner.as_str().trim().parse().unwrap();
            Expression::Literal(Literal::Float(FloatLiteral { entity, value }))
        }
        Rule::boolean => {
            let entity = ast.create_entity();
            add_position(ast, entity, &inner);
            let value = inner.as_str().trim() == "true";
            Expression::Literal(Literal::Boolean(BooleanLiteral { entity, value }))
        }
        Rule::string => parse_string_literal(ast, inner),
        Rule::array_literal => parse_array_literal(ast, inner),
        _ => todo!("Unhandled atomic value type: {:?}", inner.as_rule()),
    }
}

fn parse_string_literal(ast: &mut Ast, pair: Pair<Rule>) -> Expression {
    let entity = ast.create_entity();
    add_position(ast, entity, &pair);
    
    let mut fragments = Vec::new();
    
    if let Some(content_pair) = pair.into_inner().find(|p| p.as_rule() == Rule::string_content) {
        for inner_pair in content_pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::string_fragment => {
                    fragments.push(StringFragment::Text(inner_pair.as_str().to_string()));
                }
                Rule::string_interpolant => {
                    let expr_pair = inner_pair.into_inner().next().unwrap();
                    let expr = Expression::parse(ast, expr_pair);
                    fragments.push(StringFragment::Interpolation(expr));
                }
                _ => {}
            }
        }
    }
    
    Expression::Literal(Literal::String(StringLiteral { entity, fragments }))
}

fn parse_array_literal(ast: &mut Ast, pair: Pair<Rule>) -> Expression {
    let entity = ast.create_entity();
    add_position(ast, entity, &pair);
    
    let mut elements = Vec::new();
    
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::expression {
            elements.push(Expression::parse(ast, inner_pair));
        }
    }
    
    Expression::Literal(Literal::Array(ArrayLiteral { entity, elements }))
}

fn parse_type_expression(ast: &mut Ast, pair: Pair<Rule>) -> Expression {
    let type_expr = pair.into_inner().next().unwrap();
    
    match type_expr.as_rule() {
        Rule::integer_type => {
            let entity = ast.create_entity();
            add_position(ast, entity, &type_expr);
            Expression::Identifier(Identifier {
                entity,
                name: type_expr.as_str().to_string(),
            })
        }
        Rule::floating_type => {
            let entity = ast.create_entity();
            add_position(ast, entity, &type_expr);
            Expression::Identifier(Identifier {
                entity,
                name: type_expr.as_str().to_string(),
            })
        }
        // TODO: Handle other type expressions
        _ => {
            let entity = ast.create_entity();
            Expression::Identifier(Identifier {
                entity,
                name: "_".to_string(),
            })
        }
    }
}

fn parse_block_statements(ast: &mut Ast, block_pair: Pair<Rule>) -> Vec<Statement> {
    let mut statements = Vec::new();
    
    for inner_pair in block_pair.into_inner() {
        if inner_pair.as_rule() == Rule::decorated_statement {
            let mut visitor = ParserVisitor::new("");
            if let Some(stmt) = visitor.parse_decorated_statement(ast, inner_pair) {
                statements.push(stmt);
            }
        }
    }
    
    statements
}

#[cfg(test)]
mod inline_tests {
    use super::*;
    
    #[test]
    fn test_parse_empty() {
        let result = parse_file("");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 0);
    }
    
    #[test]
    fn test_parse_let_statement() {
        let result = parse_file("let x = 42");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Let(let_stmt) => {
                assert!(!let_stmt.is_static);
                assert!(!let_stmt.is_mutable);
                match &let_stmt.pattern {
                    Pattern::Identifier(ident) => assert_eq!(ident.name, "x"),
                    _ => panic!("Expected identifier pattern"),
                }
                assert!(let_stmt.value.is_some());
            }
            _ => panic!("Expected let statement"),
        }
    }
    
    #[test]
    fn test_parse_function() {
        let result = parse_file("fun add(x: i32, y: i32): i32 { x }");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Function(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.parameters.len(), 2);
                assert!(func.return_type.is_some());
                assert!(func.body.is_some());
            }
            _ => panic!("Expected function statement"),
        }
    }
    
    #[test]
    fn test_parse_if_expression() {
        let result = parse_file("if x { 1 } else { 2 }");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Expression(Expression::If(if_expr)) => {
                assert_eq!(if_expr.branches.len(), 1);
                assert!(if_expr.else_branch.is_some());
            }
            _ => panic!("Expected if expression"),
        }
    }
    
    #[test]
    fn test_parse_string_with_interpolation() {
        let result = parse_file(r#""Hello \(name)!""#);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Expression(Expression::Literal(Literal::String(str_lit))) => {
                assert_eq!(str_lit.fragments.len(), 3);
                assert!(matches!(str_lit.fragments[0], StringFragment::Text(_)));
                assert!(matches!(str_lit.fragments[1], StringFragment::Interpolation(_)));
                assert!(matches!(str_lit.fragments[2], StringFragment::Text(_)));
            }
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_parse_operation() {
        let result = parse_file("a + b * c");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Expression(Expression::Operation(op)) => {
                assert_eq!(op.elements.len(), 5); // a, +, b, *, c
            }
            _ => panic!("Expected operation"),
        }
    }
    
    #[test]
    fn test_parse_module() {
        let result = parse_file("module math { let pi = 3.14 }");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Module(module) => {
                assert_eq!(module.name, "math");
                assert_eq!(module.statements.len(), 1);
            }
            _ => panic!("Expected module statement"),
        }
    }
    
    #[test]
    fn test_parse_static_let() {
        let result = parse_file("static let CONSTANT = 100");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Let(let_stmt) => {
                assert!(let_stmt.is_static);
                assert!(!let_stmt.is_mutable);
            }
            _ => panic!("Expected let statement"),
        }
    }
    
    #[test]
    fn test_parse_call_chain() {
        let result = parse_file("obj.method(1, 2).field");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Expression(Expression::Field(field)) => {
                assert_eq!(field.field, "field");
                match &*field.object {
                    Expression::Call(call) => {
                        assert_eq!(call.arguments.len(), 2);
                    }
                    _ => panic!("Expected call expression"),
                }
            }
            _ => panic!("Expected field expression"),
        }
    }
    
    #[test]
    fn test_parse_lambda() {
        let result = parse_file("fun (x, y) { x + y }");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.root.statements.len(), 1);
        
        match &ast.root.statements[0] {
            Statement::Expression(Expression::Lambda(lambda)) => {
                assert_eq!(lambda.parameters.len(), 2);
                assert!(lambda.return_type.is_none());
            }
            _ => panic!("Expected lambda expression"),
        }
    }
}