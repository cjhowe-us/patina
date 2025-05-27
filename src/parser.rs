use crate::ast::*;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PatinaParser;

/// Convert pest span to our Position struct
fn span_to_position(span: &pest::Span) -> Position {
    let (line, column) = span.start_pos().line_col();
    Position {
        filename: None,
        line,
        column,
        offset: span.start(),
    }
}

/// Parse a Patina source file
pub fn parse_file(input: &str) -> Result<Root, Box<dyn std::error::Error>> {
    parse_file_with_name("", input)
}

/// Parse a Patina source file with a given filename
pub fn parse_file_with_name(
    filename: &str,
    input: &str,
) -> Result<Root, Box<dyn std::error::Error>> {
    let pairs = PatinaParser::parse(Rule::root, input)?;
    let pair = pairs.into_iter().next().unwrap();
    let mut root = parse_root(pair)?;

    root.pos.filename = Some(filename.to_string());

    Ok(root)
}

pub fn parse_root(pair: pest::iterators::Pair<Rule>) -> Result<Root, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::decorated_statement => {
                statements.push(parse_decorated_statement(inner_pair)?);
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(Root { pos, statements })
}

fn parse_decorated_statement(
    pair: pest::iterators::Pair<Rule>,
) -> Result<DecoratedStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut annotations = Vec::new();
    let mut comments = Vec::new();
    let mut statement = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::annotation => {
                annotations.push(parse_annotation(inner_pair)?);
            }
            Rule::line_comment | Rule::block_comment => {
                comments.push(parse_comment(inner_pair)?);
            }
            Rule::statement => {
                statement = Some(parse_statement(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(DecoratedStatement {
        pos: pos.clone(),
        annotations,
        comments,
        statement: statement.unwrap_or(StatementKind::Expression(ExpressionStatement {
            pos: pos.clone(),
            expression: ExpressionKind::Block(BlockExpression {
                pos: pos.clone(),
                is_static: false,
                block: Block {
                    pos,
                    label: None,
                    statements: Vec::new(),
                },
            }),
        })),
    })
}

fn parse_annotation(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Annotation, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let name = pair.as_str().to_string();
    Ok(Annotation {
        pos,
        name,
        arguments: Vec::new(),
    })
}

fn parse_comment(pair: pest::iterators::Pair<Rule>) -> Result<Comment, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let text = Some(pair.as_str().to_string());
    Ok(Comment {
        pos,
        text,
        nested_comments: Vec::new(),
    })
}

fn parse_statement(
    pair: pest::iterators::Pair<Rule>,
) -> Result<StatementKind, Box<dyn std::error::Error>> {
    // Get position before consuming the pair
    let default_pos = span_to_position(&pair.as_span());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::module_stmt => {
                return Ok(StatementKind::Module(parse_module(inner_pair)?));
            }
            Rule::function_stmt => {
                return Ok(StatementKind::Function(parse_function(inner_pair)?));
            }
            Rule::return_stmt => {
                return Ok(StatementKind::Return(parse_return(inner_pair)?));
            }
            Rule::let_stmt => {
                return Ok(StatementKind::Let(parse_let(inner_pair)?));
            }
            Rule::assignment_stmt => {
                return Ok(StatementKind::Assignment(parse_assignment(inner_pair)?));
            }
            Rule::break_stmt => {
                return Ok(StatementKind::Break(parse_break(inner_pair)?));
            }
            Rule::while_stmt => {
                return Ok(StatementKind::While(parse_while(inner_pair)?));
            }
            Rule::expression => {
                return Ok(StatementKind::Expression(ExpressionStatement {
                    pos: span_to_position(&inner_pair.as_span()),
                    expression: parse_expression(inner_pair)?,
                }));
            }
            _ => {} // Skip other rules
        }
    }

    // Default fallback - using the position we captured earlier
    let pos = default_pos;
    Ok(StatementKind::Expression(ExpressionStatement {
        pos: pos.clone(),
        expression: ExpressionKind::Block(BlockExpression {
            pos: pos.clone(),
            is_static: false,
            block: Block {
                pos,
                label: None,
                statements: Vec::new(),
            },
        }),
    }))
}

fn parse_module(pair: pest::iterators::Pair<Rule>) -> Result<Module, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut name = String::new();
    let mut body = Block {
        pos: Position::default(),
        label: None,
        statements: Vec::new(),
    };

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => {
                name = inner_pair.as_str().to_string();
            }
            Rule::block => {
                body = parse_block(inner_pair)?;
            }
            _ => {} // Skip other rules
        }
    }

    Ok(Module { pos, name, body })
}

fn parse_function(
    pair: pest::iterators::Pair<Rule>,
) -> Result<FunctionStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut name = String::new();
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = None;
    let mut is_static = false;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::static_kw => {
                is_static = true;
            }
            Rule::ident => {
                name = inner_pair.as_str().to_string();
            }
            Rule::function_parameter => {
                parameters.push(parse_function_parameter(inner_pair)?);
            }
            Rule::atom => {
                return_type = Some(parse_atom(inner_pair)?);
            }
            Rule::block => {
                body = Some(parse_block(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(FunctionStatement {
        pos,
        is_static,
        name,
        parameters,
        return_type,
        block: body.unwrap_or(Block {
            pos: Position::default(),
            label: None,
            statements: Vec::new(),
        }),
    })
}

fn parse_function_parameter(
    pair: pest::iterators::Pair<Rule>,
) -> Result<FunctionParameter, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut name = String::new();
    let mut param_type = AtomKind::Value(AtomicValue {
        pos: pos.clone(),
        value: None,
    });

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => {
                name = inner_pair.as_str().to_string();
            }
            Rule::atom => {
                param_type = parse_atom(inner_pair)?;
            }
            _ => {} // Skip other rules
        }
    }

    Ok(FunctionParameter {
        pos,
        name,
        param_type,
    })
}

fn parse_return(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ReturnStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut label = None;
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label => {
                // Extract the label without the leading colon
                let label_str = inner_pair.as_str();
                label = Some(label_str.trim_start_matches(':').to_string());
            }
            Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(ReturnStatement { pos, label, value })
}

fn parse_let(
    pair: pest::iterators::Pair<Rule>,
) -> Result<LetStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut name = String::new();
    let mut var_type = None;
    let mut value = None;
    let mut is_mutable = false;
    let mut is_static = false;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::static_kw => {
                is_static = true;
            }
            Rule::mut_kw => {
                is_mutable = true;
            }
            Rule::ident => {
                name = inner_pair.as_str().to_string();
            }
            Rule::atomic_type => {
                var_type = Some(AtomKind::Type(parse_atomic_type(inner_pair)?));
            }
            Rule::atom => {
                var_type = Some(parse_atom(inner_pair)?);
            }
            Rule::operator => {
                // Skip the = operator
            }
            Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(LetStatement {
        pos,
        is_static,
        is_mutable,
        name,
        var_type,
        value,
    })
}

fn parse_assignment(
    pair: pest::iterators::Pair<Rule>,
) -> Result<AssignmentStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut target = None;
    let mut value = None;

    let expressions: Vec<_> = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::expression)
        .collect();

    if expressions.len() >= 2 {
        target = Some(parse_expression(expressions[0].clone())?);
        value = Some(parse_expression(expressions[1].clone())?);
    }

    Ok(AssignmentStatement {
        pos: pos.clone(),
        target: target.unwrap_or(ExpressionKind::Block(BlockExpression {
            pos: pos.clone(),
            is_static: false,
            block: Block {
                pos: pos.clone(),
                label: None,
                statements: Vec::new(),
            },
        })),
        value: value.unwrap_or(ExpressionKind::Block(BlockExpression {
            pos: pos.clone(),
            is_static: false,
            block: Block {
                pos,
                label: None,
                statements: Vec::new(),
            },
        })),
    })
}

fn parse_break(
    pair: pest::iterators::Pair<Rule>,
) -> Result<BreakStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut label = None;
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label => {
                // Extract the label without the leading colon
                let label_str = inner_pair.as_str();
                label = Some(label_str.trim_start_matches(':').to_string());
            }
            Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(BreakStatement { pos, label, value })
}

fn parse_while(
    pair: pest::iterators::Pair<Rule>,
) -> Result<WhileStatement, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut condition = None;
    let mut body = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                condition = Some(parse_expression(inner_pair)?);
            }
            Rule::block => {
                body = Some(parse_block(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(WhileStatement {
        pos,
        condition: condition.unwrap_or(ExpressionKind::Block(BlockExpression {
            pos: Position::default(),
            is_static: false,
            block: Block {
                pos: Position::default(),
                label: None,
                statements: Vec::new(),
            },
        })),
        body: body.unwrap_or(Block {
            pos: Position::default(),
            label: None,
            statements: Vec::new(),
        }),
    })
}

fn parse_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ExpressionKind, Box<dyn std::error::Error>> {
    // Get position before consuming the pair
    let default_pos = span_to_position(&pair.as_span());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::if_expression => {
                return Ok(ExpressionKind::If(parse_if_expression(inner_pair)?));
            }
            Rule::case_expression => {
                return Ok(ExpressionKind::Case(parse_case_expression(inner_pair)?));
            }
            Rule::lambda => {
                return Ok(ExpressionKind::Lambda(parse_lambda(inner_pair)?));
            }
            Rule::block => {
                return Ok(ExpressionKind::Block(parse_block_expression(inner_pair)?));
            }
            Rule::operation => {
                return Ok(ExpressionKind::Operation(parse_operation(inner_pair)?));
            }
            _ => {} // Skip other rules
        }
    }

    // Default fallback
    let pos = default_pos;
    Ok(ExpressionKind::Block(BlockExpression {
        pos: pos.clone(),
        is_static: false,
        block: Block {
            pos,
            label: None,
            statements: Vec::new(),
        },
    }))
}

fn parse_if_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<IfExpression, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut if_branch = None;
    let mut else_ifs = Vec::new();
    let mut else_branch = None;

    let inner_pairs: Vec<_> = pair.into_inner().collect();

    // The first if_branch is the main if
    if let Some(first_pair) = inner_pairs.get(0) {
        if first_pair.as_rule() == Rule::if_branch {
            if_branch = Some(parse_if_branch(first_pair.clone())?);
        }
    }

    // Process remaining pairs
    let mut i = 1;
    while i < inner_pairs.len() {
        let pair = &inner_pairs[i];

        match pair.as_rule() {
            Rule::else_kw => {
                // Check what follows else
                if i + 1 < inner_pairs.len() {
                    let next_pair = &inner_pairs[i + 1];
                    match next_pair.as_rule() {
                        Rule::if_expression => {
                            // This is an else-if
                            let else_if = parse_if_expression(next_pair.clone())?;
                            if let Some(branch) = else_if.if_branch {
                                else_ifs.push(branch);
                            }
                            i += 1; // Skip the if_expression we just processed
                        }
                        Rule::block => {
                            // This is an else block
                            let block = parse_block(next_pair.clone())?;
                            else_branch = Some(Box::new(ExpressionKind::Block(BlockExpression {
                                pos: block.pos.clone(),
                                is_static: false,
                                block,
                            })));
                            i += 1; // Skip the block we just processed
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    Ok(IfExpression {
        pos,
        if_branch,
        else_ifs,
        else_branch,
    })
}

fn parse_if_branch(
    pair: pest::iterators::Pair<Rule>,
) -> Result<IfBranch, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut condition_expr = None;
    let mut then_block = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                // The expression is the condition
                condition_expr = Some(Box::new(parse_expression(inner_pair)?));
            }
            Rule::block => {
                // The block is the then part
                let block = parse_block(inner_pair)?;
                then_block = Some(Box::new(ExpressionKind::Block(BlockExpression {
                    pos: block.pos.clone(),
                    is_static: false,
                    block,
                })));
            }
            _ => {} // Skip other rules
        }
    }

    // Create a cloned position that won't be moved
    let default_pos = pos.clone();

    // Default empty expressions if not provided - use the cloned position
    let default_expr = || {
        Box::new(ExpressionKind::Block(BlockExpression {
            pos: default_pos.clone(),
            is_static: false,
            block: Block {
                pos: default_pos.clone(),
                label: None,
                statements: Vec::new(),
            },
        }))
    };

    Ok(IfBranch {
        pos: default_pos.clone(), // Use the cloned position
        condition: condition_expr.unwrap_or_else(default_expr),
        then: then_block.unwrap_or_else(default_expr),
    })
}

fn parse_case_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<CaseExpression, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut branches = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::case_branch {
            branches.push(parse_case_branch(inner_pair)?);
        }
    }

    Ok(CaseExpression { pos, branches })
}

fn parse_case_branch(
    pair: pest::iterators::Pair<Rule>,
) -> Result<CaseBranch, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut pattern = None;
    let mut then = None;

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::expression {
            if pattern.is_none() {
                pattern = Some(Box::new(parse_expression(inner_pair)?));
            } else {
                then = Some(Box::new(parse_expression(inner_pair)?));
            }
        }
    }

    Ok(CaseBranch {
        pos: pos.clone(),
        pattern,
        then: then.unwrap_or_else(|| {
            Box::new(ExpressionKind::Block(BlockExpression {
                pos: pos.clone(),
                is_static: false,
                block: Block {
                    pos,
                    label: None,
                    statements: Vec::new(),
                },
            }))
        }),
    })
}

fn parse_lambda(pair: pest::iterators::Pair<Rule>) -> Result<Lambda, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::function_parameter => {
                parameters.push(parse_function_parameter(inner_pair)?);
            }
            Rule::atomic_type => {
                return_type = Some(Box::new(AtomKind::Type(parse_atomic_type(inner_pair)?)));
            }
            Rule::block => {
                body = Some(parse_block(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(Lambda {
        pos: pos.clone(),
        parameters,
        return_type,
        body: body.unwrap_or(Block {
            pos,
            label: None,
            statements: Vec::new(),
        }),
    })
}

fn parse_block_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<BlockExpression, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let block = parse_block(pair)?;
    Ok(BlockExpression {
        pos,
        is_static: false,
        block,
    })
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Block, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut statements = Vec::new();
    let mut label = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label => {
                // Extract the label without the leading colon
                let label_str = inner_pair.as_str();
                label = Some(label_str.trim_start_matches(':').to_string());
            }
            Rule::decorated_statement => {
                statements.push(parse_decorated_statement(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(Block {
        pos,
        label,
        statements,
    })
}

fn parse_operation(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Operation, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut operands = Vec::new();
    let mut operators = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::operand => {
                operands.push(parse_operand(inner_pair)?);
            }
            Rule::operator => {
                operators.push(inner_pair.as_str().to_string());
            }
            _ => {} // Skip other rules
        }
    }

    // Build the operation tree (for now just return the first operand wrapped)
    if operands.is_empty() {
        Ok(Operation {
            pos,
            prefix: Vec::new(),
            operands: vec![Operand {
                pos: Position::default(),
                atom: AtomKind::Value(AtomicValue {
                    pos: Position::default(),
                    value: None,
                }),
                postfix: Vec::new(),
            }],
        })
    } else {
        Ok(Operation {
            pos,
            prefix: operators.clone(),
            operands,
        })
    }
}

fn parse_operand(pair: pest::iterators::Pair<Rule>) -> Result<Operand, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut atom = AtomKind::Value(AtomicValue {
        pos: pos.clone(),
        value: None,
    });
    let mut postfix = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::atom => {
                atom = parse_atom(inner_pair)?;
            }
            Rule::postfix => {
                postfix.push(parse_postfix(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(Operand { pos, atom, postfix })
}

fn parse_postfix(pair: pest::iterators::Pair<Rule>) -> Result<Postfix, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let parent_str = pair.as_str().to_string(); // Get string before consuming pair
    let mut invocation = Vec::new();
    let mut index = None;
    let mut field = None;
    let mut operator = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                // Check parent context to determine if this is invocation or index
                if parent_str.starts_with('(') {
                    // Function invocation
                    invocation.push(parse_expression(inner_pair)?);
                } else if parent_str.starts_with('[') {
                    // Array index
                    index = Some(Box::new(parse_expression(inner_pair)?));
                }
            }
            Rule::field => {
                // Handle field access - extract the identifier after the dot
                for field_inner in inner_pair.into_inner() {
                    if field_inner.as_rule() == Rule::ident {
                        field = Some(field_inner.as_str().to_string());
                    }
                }
            }
            Rule::operator => {
                operator = Some(inner_pair.as_str().to_string());
            }
            Rule::ident => {
                // Direct identifier in field context
                field = Some(inner_pair.as_str().to_string());
            }
            _ => {} // Skip other rules
        }
    }

    Ok(Postfix {
        pos,
        invocation,
        index,
        field,
        operator,
    })
}

fn parse_atom(pair: pest::iterators::Pair<Rule>) -> Result<AtomKind, Box<dyn std::error::Error>> {
    // Get position before consuming pair
    let default_pos = span_to_position(&pair.as_span());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::grouped_expression => {
                return Ok(AtomKind::Grouped(parse_grouped_expression(inner_pair)?));
            }
            Rule::atomic_type => {
                return Ok(AtomKind::Type(parse_atomic_type(inner_pair)?));
            }
            Rule::atomic_value => {
                return Ok(AtomKind::Value(parse_atomic_value(inner_pair)?));
            }
            Rule::binding => {
                return Ok(AtomKind::Binding(parse_binding(inner_pair)?));
            }
            _ => {} // Skip other rules
        }
    }

    // Default fallback
    Ok(AtomKind::Value(AtomicValue {
        pos: default_pos,
        value: None,
    }))
}

fn parse_grouped_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<GroupedExpression, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut expression = None;

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::expression {
            expression = Some(Box::new(parse_expression(inner_pair)?));
        }
    }

    Ok(GroupedExpression { pos, expression })
}

fn parse_atomic_type(
    pair: pest::iterators::Pair<Rule>,
) -> Result<AtomicType, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut type_expr = None;

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::type_expression {
            type_expr = Some(Box::new(parse_type_expression(inner_pair)?));
        }
    }

    Ok(AtomicType { pos, type_expr })
}

fn parse_type_expression(
    pair: pest::iterators::Pair<Rule>,
) -> Result<TypeKind, Box<dyn std::error::Error>> {
    // Get position before consuming the pair
    let default_pos = span_to_position(&pair.as_span());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::pointer_type => {
                return Ok(TypeKind::Pointer(parse_pointer_type(inner_pair)?));
            }
            Rule::array_type => {
                return Ok(TypeKind::Array(parse_array_type(inner_pair)?));
            }
            Rule::integer_type => {
                return Ok(TypeKind::Integer(parse_integer_type(inner_pair)?));
            }
            Rule::floating_type => {
                return Ok(TypeKind::Floating(parse_floating_type(inner_pair)?));
            }
            Rule::function_type => {
                return Ok(TypeKind::Function(parse_function_type(inner_pair)?));
            }
            _ => {} // Skip other rules
        }
    }

    // Default fallback
    Ok(TypeKind::Integer(IntegerType {
        pos: default_pos,
        bit_size: 32,
        signed: true,
    }))
}

fn parse_pointer_type(
    pair: pest::iterators::Pair<Rule>,
) -> Result<PointerType, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut target = None;
    let mut is_owned = false;
    let mut is_raw = false;
    let mut is_mutable = false;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::own_kw => {
                is_owned = true;
            }
            Rule::raw_kw => {
                is_raw = true;
            }
            Rule::mut_kw => {
                is_mutable = true;
            }
            Rule::atom => {
                target = Some(parse_atom(inner_pair)?);
            }
            _ => {} // Skip other rules
        }
    }

    Ok(PointerType {
        pos: pos.clone(),
        is_owned,
        is_raw,
        is_mutable,
        target: target.unwrap_or(AtomKind::Value(AtomicValue { pos, value: None })),
    })
}

fn parse_array_type(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ArrayType, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut element = None;
    let mut dimension = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::atomic_type => {
                element = Some(AtomKind::Type(parse_atomic_type(inner_pair)?));
            }
            Rule::integer => {
                let value_str = inner_pair.as_str();
                // Since we can have negative array dimensions in tests, use i64 instead
                // This only affects the parser, not semantics validation
                if let Ok(parsed_value) = value_str.parse::<i64>() {
                    if parsed_value >= 0 {
                        dimension = Some(parsed_value as u64);
                    }
                    // else invalid dimension - maintain none
                }
            }
            _ => {} // Skip other rules
        }
    }

    Ok(ArrayType {
        pos: pos.clone(),
        element: element.unwrap_or(AtomKind::Value(AtomicValue { pos, value: None })),
        dimension,
    })
}

fn parse_integer_type(
    pair: pest::iterators::Pair<Rule>,
) -> Result<IntegerType, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let type_str = pair.as_str();

    // Parse i32, u64, etc.
    let signed = type_str.starts_with('i');
    let bit_size = type_str[1..].parse::<u64>().unwrap_or(32);

    Ok(IntegerType {
        pos,
        bit_size,
        signed,
    })
}

fn parse_floating_type(
    pair: pest::iterators::Pair<Rule>,
) -> Result<FloatingType, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let type_str = pair.as_str();

    // Parse f32, f64, etc.
    let bit_size = type_str[1..].parse::<u64>().unwrap_or(64);

    Ok(FloatingType { pos, bit_size })
}

fn parse_function_type(
    pair: pest::iterators::Pair<Rule>,
) -> Result<FunctionType, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut parameters = Vec::new();
    let mut return_type = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::function_parameter => {
                parameters.push(parse_function_parameter(inner_pair)?);
            }
            Rule::atomic_type => {
                return_type = Some(AtomKind::Type(parse_atomic_type(inner_pair)?));
            }
            _ => {} // Skip other rules
        }
    }

    Ok(FunctionType {
        pos,
        name: None,
        parameters,
        return_type,
    })
}

fn parse_atomic_value(
    pair: pest::iterators::Pair<Rule>,
) -> Result<AtomicValue, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::integer => {
                value = Some(ValueKind::Integer(parse_integer(inner_pair)?));
            }
            Rule::float => {
                value = Some(ValueKind::Float(parse_float(inner_pair)?));
            }
            Rule::boolean => {
                value = Some(ValueKind::Boolean(parse_boolean(inner_pair)?));
            }
            Rule::string => {
                value = Some(ValueKind::String(parse_string(inner_pair)?));
            }
            _ => {} // Skip other rules
        }
    }

    Ok(AtomicValue { pos, value })
}

fn parse_integer(pair: pest::iterators::Pair<Rule>) -> Result<Integer, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let value_str = pair.as_str();
    let value = value_str.parse::<u64>().unwrap_or(0);
    Ok(Integer { pos, value })
}

fn parse_float(pair: pest::iterators::Pair<Rule>) -> Result<Float, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let value_str = pair.as_str();
    let value = value_str.parse::<f64>().unwrap_or(0.0);
    Ok(Float { pos, value })
}

fn parse_boolean(pair: pest::iterators::Pair<Rule>) -> Result<Boolean, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let value = match pair.as_str() {
        "true" => true,
        "false" => false,
        _ => false, // default to false for unexpected values
    };
    Ok(Boolean { pos, value })
}

fn parse_string(
    pair: pest::iterators::Pair<Rule>,
) -> Result<StringLiteral, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut fragments = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::string_content => {
                // Parse the string content which contains fragments and interpolants
                for content_pair in inner_pair.into_inner() {
                    match content_pair.as_rule() {
                        Rule::string_fragment => {
                            fragments.push(parse_string_fragment(content_pair)?);
                        }
                        Rule::string_interpolant => {
                            fragments.push(parse_string_interpolant(content_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(StringLiteral { pos, fragments })
}

fn parse_string_fragment(
    pair: pest::iterators::Pair<Rule>,
) -> Result<StringFragment, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let text = Some(pair.as_str().to_string());

    Ok(StringFragment {
        pos,
        text,
        interpolation: None,
    })
}

fn parse_string_interpolant(
    pair: pest::iterators::Pair<Rule>,
) -> Result<StringFragment, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let mut interpolation = None;

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::expression {
            interpolation = Some(parse_expression(inner_pair)?);
        }
    }

    Ok(StringFragment {
        pos,
        text: None,
        interpolation,
    })
}

fn parse_binding(pair: pest::iterators::Pair<Rule>) -> Result<Binding, Box<dyn std::error::Error>> {
    let pos = span_to_position(&pair.as_span());
    let name = pair.as_str().to_string();
    Ok(Binding { pos, name })
}

#[cfg(test)]
mod tests;
