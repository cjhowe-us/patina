use pest::Parser;
use pest::iterators::Pair;
use crate::parser::{PatinaParser, Rule};
use crate::ast_ecs::{AstWorld, builder::AstBuilder, components::*};
use hecs::Entity;

pub fn parse_ecs(input: &str) -> Result<AstWorld, pest::error::Error<Rule>> {
    let mut pairs = PatinaParser::parse(Rule::root, input)?;
    let root_pair = pairs.next().unwrap();
    
    let mut ast_world = AstWorld::new();
    let mut builder = AstBuilder::from_ast_world(&mut ast_world);
    
    let root_entity = parse_root(&mut builder, root_pair);
    ast_world.set_root(root_entity);
    
    Ok(ast_world)
}

fn parse_root(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let mut statements = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::decorated_statement => {
                statements.push(parse_decorated_statement(builder, inner));
            }
            Rule::EOI => {}
            _ => unreachable!("Unexpected rule in root: {:?}", inner.as_rule()),
        }
    }
    
    builder.create_root(statements)
}

fn parse_decorated_statement(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut comments = Vec::new();
    let mut annotations = Vec::new();
    let mut statement = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::comment => {
                comments.push(parse_comment(builder, inner));
            }
            Rule::annotation => {
                annotations.push(parse_annotation(builder, inner));
            }
            Rule::statement => {
                statement = Some(parse_statement(builder, inner));
            }
            _ => {}
        }
    }
    
    let statement_entity = statement.expect("Decorated statement must have a statement");
    builder.create_decorated_statement(&span, comments, annotations, statement_entity)
}

fn parse_comment(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let text = pair.as_str().to_string();
    let is_doc = text.starts_with("///") || text.starts_with("//!");
    builder.create_comment(&span, text, is_doc)
}

fn parse_annotation(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut name = String::new();
    let mut arguments = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::expression => {
                arguments.push(parse_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    builder.create_annotation(&span, name, arguments)
}

fn parse_statement(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::module_statement => parse_module(builder, inner),
        Rule::function_statement => parse_function(builder, inner),
        Rule::let_statement => parse_let(builder, inner),
        Rule::return_statement => parse_return(builder, inner),
        Rule::break_statement => parse_break(builder, inner),
        Rule::while_statement => parse_while(builder, inner),
        Rule::expression_statement => parse_expression_statement(builder, inner),
        _ => unreachable!("Unexpected statement rule: {:?}", inner.as_rule()),
    }
}

fn parse_module(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut name = String::new();
    let mut declarations = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::block => {
                declarations = parse_block_statements(builder, inner);
            }
            _ => {}
        }
    }
    
    builder.create_module(&span, name, declarations)
}

fn parse_function(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut is_static = false;
    let mut name = String::new();
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::static_keyword => {
                is_static = true;
            }
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::function_parameters => {
                parameters = parse_function_parameters(builder, inner);
            }
            Rule::atom => {
                return_type = Some(parse_atom(builder, inner));
            }
            Rule::block => {
                body = Some(parse_block_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    builder.create_function(&span, name, is_static, parameters, return_type, body)
}

fn parse_function_parameters(builder: &mut AstBuilder, pair: Pair<Rule>) -> Vec<Entity> {
    let mut parameters = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::function_parameter {
            parameters.push(parse_function_parameter(builder, inner));
        }
    }
    
    parameters
}

fn parse_function_parameter(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut name = String::new();
    let mut param_type = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::atom => {
                param_type = Some(parse_atom(builder, inner));
            }
            _ => {}
        }
    }
    
    let type_entity = param_type.expect("Function parameter must have a type");
    builder.create_function_parameter(&span, name, type_entity)
}

fn parse_let(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut is_static = false;
    let mut is_mutable = false;
    let mut binding_path = vec![];
    let mut type_ref = None;
    let mut value = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::static_keyword => {
                is_static = true;
            }
            Rule::mut_keyword => {
                is_mutable = true;
            }
            Rule::binding_path => {
                binding_path = parse_binding_path(inner);
            }
            Rule::atom => {
                type_ref = Some(parse_atom(builder, inner));
            }
            Rule::expression => {
                value = Some(parse_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    let binding = builder.create_binding(&span, binding_path);
    builder.create_let_statement(&span, is_static, is_mutable, binding, type_ref, value)
}

fn parse_binding_path(pair: Pair<Rule>) -> Vec<String> {
    let mut path = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::ident {
            path.push(inner.as_str().to_string());
        }
    }
    
    path
}

fn parse_return(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut label = None;
    let mut value = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::label => {
                label = Some(inner.as_str().trim_start_matches(':').to_string());
            }
            Rule::expression => {
                value = Some(parse_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    builder.create_return_statement(&span, label, value)
}

fn parse_break(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut label = None;
    let mut value = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::label => {
                label = Some(inner.as_str().trim_start_matches(':').to_string());
            }
            Rule::expression => {
                value = Some(parse_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    builder.create_break_statement(&span, label, value)
}

fn parse_while(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut condition = None;
    let mut body = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => {
                condition = Some(parse_expression(builder, inner));
            }
            Rule::block => {
                body = Some(parse_block_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    // Create a while loop entity (using root as placeholder for now)
    let children = vec![
        condition.expect("While must have condition"),
        body.expect("While must have body"),
    ];
    builder.create_root(children)
}

fn parse_expression_statement(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let inner = pair.into_inner().next().unwrap();
    parse_expression(builder, inner)
}

fn parse_expression(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::if_expression => parse_if_expression(builder, inner),
        Rule::lambda => parse_lambda(builder, inner),
        Rule::block_expression => parse_block_expression(builder, inner),
        Rule::operation => parse_operation(builder, inner),
        _ => unreachable!("Unexpected expression rule: {:?}", inner.as_rule()),
    }
}

fn parse_if_expression(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut branches = Vec::new();
    let is_expression = true; // For now, all ifs are expressions
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::if_branch => {
                branches.push(parse_if_branch(builder, inner, true));
            }
            Rule::else_if_branch => {
                branches.push(parse_if_branch(builder, inner, true));
            }
            Rule::else_branch => {
                branches.push(parse_if_branch(builder, inner, false));
            }
            _ => {}
        }
    }
    
    builder.create_if_expression(&span, is_expression, branches)
}

fn parse_if_branch(builder: &mut AstBuilder, pair: Pair<Rule>, has_condition: bool) -> Entity {
    let span = pair.as_span();
    let mut condition = None;
    let mut consequent = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => {
                if has_condition && condition.is_none() {
                    condition = Some(parse_expression(builder, inner));
                } else {
                    consequent = Some(parse_expression(builder, inner));
                }
            }
            Rule::block => {
                consequent = Some(parse_block_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    let consequent_entity = consequent.expect("If branch must have consequent");
    builder.create_if_branch(&span, condition, consequent_entity)
}

fn parse_lambda(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::function_parameters => {
                parameters = parse_function_parameters(builder, inner);
            }
            Rule::atom => {
                return_type = Some(parse_atom(builder, inner));
            }
            Rule::expression => {
                body = Some(parse_expression(builder, inner));
            }
            _ => {}
        }
    }
    
    let body_entity = body.expect("Lambda must have body");
    builder.create_lambda(&span, parameters, return_type, body_entity)
}

fn parse_block_expression(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut label = None;
    let mut is_static = false;
    let mut statements = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::label => {
                label = Some(inner.as_str().trim_start_matches(':').to_string());
            }
            Rule::static_keyword => {
                is_static = true;
            }
            Rule::decorated_statement => {
                statements.push(parse_decorated_statement(builder, inner));
            }
            _ => {}
        }
    }
    
    builder.create_block_expression(&span, label, is_static, statements)
}

fn parse_block_statements(builder: &mut AstBuilder, pair: Pair<Rule>) -> Vec<Entity> {
    let mut statements = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::decorated_statement {
            statements.push(parse_decorated_statement(builder, inner));
        }
    }
    
    statements
}

fn parse_operation(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut operator = String::new();
    let mut left = None;
    let mut right = None;
    let mut operand = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::infix_op => {
                operator = inner.as_str().to_string();
            }
            Rule::prefix_op => {
                operator = inner.as_str().to_string();
            }
            Rule::expression => {
                if operator.is_empty() {
                    // First expression is operand for prefix
                    operand = Some(parse_expression(builder, inner));
                } else if left.is_none() {
                    left = Some(parse_expression(builder, inner));
                } else {
                    right = Some(parse_expression(builder, inner));
                }
            }
            Rule::atom => {
                if operator.is_empty() {
                    operand = Some(parse_atom(builder, inner));
                } else if left.is_none() {
                    left = Some(parse_atom(builder, inner));
                } else {
                    right = Some(parse_atom(builder, inner));
                }
            }
            _ => {}
        }
    }
    
    // Convert atom operands to entities if needed
    let left_entity = left.or(operand);
    let right_entity = right;
    let operand_entity = if left.is_some() { None } else { operand };
    
    builder.create_operation(&span, operator, left_entity, right_entity, operand_entity)
}

fn parse_atom(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::grouped_expression => parse_grouped_expression(builder, inner),
        Rule::type_atom => parse_type_atom(builder, inner),
        Rule::value_atom => parse_value_atom(builder, inner),
        Rule::binding => parse_binding(builder, inner),
        _ => unreachable!("Unexpected atom rule: {:?}", inner.as_rule()),
    }
}

fn parse_grouped_expression(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut expression = None;
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            expression = Some(parse_expression(builder, inner));
        }
    }
    
    let expr_entity = expression.expect("Grouped expression must have expression");
    builder.create_grouped_expression(&span, expr_entity)
}

fn parse_type_atom(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let type_entity = parse_type(builder, pair.into_inner().next().unwrap());
    builder.create_type_atom(&span, type_entity)
}

fn parse_value_atom(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut value = None;
    let mut postfix_ops = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::value => {
                value = Some(parse_value(builder, inner));
            }
            Rule::postfix => {
                postfix_ops.push(parse_postfix(builder, inner));
            }
            _ => {}
        }
    }
    
    let value_entity = value.expect("Value atom must have value");
    let base = builder.create_value_atom(&span, value_entity);
    
    if postfix_ops.is_empty() {
        base
    } else {
        builder.create_postfix_op(base, postfix_ops)
    }
}

fn parse_binding(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let path = parse_binding_path(pair);
    builder.create_binding(&span, path)
}

fn parse_postfix<'a>(builder: &mut AstBuilder, pair: Pair<'a, Rule>) -> (pest::Span<'a>, PostfixOpKind) {
    let span = pair.as_span();
    let mut op_kind = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::invocation => {
                let args = parse_invocation_args(builder, inner);
                op_kind = Some(PostfixOpKind::Invocation(args));
            }
            Rule::index => {
                let index_expr = parse_expression(builder, inner.into_inner().next().unwrap());
                op_kind = Some(PostfixOpKind::Index(index_expr));
            }
            Rule::postfix_op => {
                let operator = inner.as_str().to_string();
                op_kind = Some(PostfixOpKind::Operator(operator));
            }
            _ => {}
        }
    }
    
    (span, op_kind.expect("Postfix must have operation"))
}

fn parse_invocation_args(builder: &mut AstBuilder, pair: Pair<Rule>) -> Vec<Entity> {
    let mut args = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            args.push(parse_expression(builder, inner));
        }
    }
    
    args
}

fn parse_type(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    match pair.as_rule() {
        Rule::pointer_type => parse_pointer_type(builder, pair),
        Rule::array_type => parse_array_type(builder, pair),
        Rule::integer_type => parse_integer_type(builder, pair),
        Rule::function_type => parse_function_type(builder, pair),
        _ => unreachable!("Unexpected type rule: {:?}", pair.as_rule()),
    }
}

fn parse_pointer_type(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut is_owned = false;
    let mut is_raw = false;
    let mut is_mutable = false;
    let mut target = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::own_modifier => is_owned = true,
            Rule::raw_modifier => is_raw = true,
            Rule::mut_modifier => is_mutable = true,
            Rule::atom => {
                target = Some(parse_atom(builder, inner));
            }
            _ => {}
        }
    }
    
    let target_entity = target.expect("Pointer must have target type");
    builder.create_pointer_type(&span, target_entity, is_owned, is_raw, is_mutable)
}

fn parse_array_type(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut element_type = None;
    let mut dimension = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::atom => {
                element_type = Some(parse_atom(builder, inner));
            }
            Rule::integer_literal => {
                dimension = Some(inner.as_str().parse::<u64>().unwrap());
            }
            _ => {}
        }
    }
    
    let element = element_type.expect("Array must have element type");
    builder.create_array_type(&span, element, dimension)
}

fn parse_integer_type(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let type_str = pair.as_str();
    
    let is_signed = !type_str.starts_with('u');
    let bits = if type_str.contains("8") { Some(8) }
              else if type_str.contains("16") { Some(16) }
              else if type_str.contains("32") { Some(32) }
              else if type_str.contains("64") { Some(64) }
              else if type_str.contains("128") { Some(128) }
              else { None };
    
    builder.create_integer_type(&span, is_signed, bits)
}

fn parse_function_type(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut name = None;
    let mut parameters = Vec::new();
    let mut return_type = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = Some(inner.as_str().to_string());
            }
            Rule::function_parameters => {
                parameters = parse_function_parameters(builder, inner);
            }
            Rule::atom => {
                return_type = Some(parse_atom(builder, inner));
            }
            _ => {}
        }
    }
    
    builder.create_function_type(&span, name, parameters, return_type)
}

fn parse_value(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::integer_literal => {
            let span = inner.as_span();
            let value = inner.as_str().parse::<u64>().unwrap();
            builder.create_integer_value(&span, value)
        }
        Rule::boolean_literal => {
            let span = inner.as_span();
            let value = inner.as_str() == "true";
            builder.create_boolean_value(&span, value)
        }
        Rule::string_literal => {
            parse_string_literal(builder, inner)
        }
        _ => unreachable!("Unexpected value rule: {:?}", inner.as_rule()),
    }
}

fn parse_string_literal(builder: &mut AstBuilder, pair: Pair<Rule>) -> Entity {
    let span = pair.as_span();
    let mut fragments = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_fragment => {
                let content = inner.as_str().to_string();
                fragments.push(builder.create_string_fragment(&inner.as_span(), content));
            }
            Rule::string_interpolation => {
                let expr = parse_expression(builder, inner.into_inner().next().unwrap());
                fragments.push(builder.create_string_interpolation(&inner.as_span(), expr));
            }
            _ => {}
        }
    }
    
    builder.create_string_value(&span, fragments)
}