use crate::ast::*;
use pest::Parser;
use pest_derive::Parser;
use std::error::Error;
use std::fmt;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PatinaParser;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl Error for ParseError {}

type ParseResult<T> = Result<T, ParseError>;

fn parse_error(msg: &str) -> ParseError {
    ParseError {
        message: msg.to_string(),
    }
}

fn parse_position(pair: &pest::iterators::Pair<Rule>) -> Position {
    let span = pair.as_span();
    Position {
        start_line: span.start_pos().line_col().0,
        start_column: span.start_pos().line_col().1,
        end_line: span.end_pos().line_col().0,
        end_column: span.end_pos().line_col().1,
    }
}

pub fn parse_string(input: &str) -> ParseResult<Root> {
    let parsed = PatinaParser::parse(Rule::root, input)
        .map_err(|e| parse_error(&format!("Pest parsing failed: {}", e)))?;

    let root_pair = parsed
        .into_iter()
        .next()
        .ok_or_else(|| parse_error("No root found"))?;

    parse_root(root_pair)
}

fn parse_root(pair: pest::iterators::Pair<Rule>) -> ParseResult<Root> {
    let pos = parse_position(&pair);
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::decorated_statement {
            statements.push(parse_decorated_statement(inner_pair)?);
        }
    }

    Ok(Root { pos, statements })
}

fn parse_decorated_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<DecoratedStatement> {
    let pos = parse_position(&pair);
    let mut comments = Vec::new();
    let mut annotations = Vec::new();
    let mut statement = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::line_comment => {
                comments.push(parse_comment(inner_pair)?);
            }
            Rule::annotation => {
                annotations.push(parse_annotation(inner_pair)?);
            }
            Rule::statement => {
                statement = Some(parse_statement(inner_pair)?);
            }
            _ => {}
        }
    }

    let statement =
        statement.ok_or_else(|| parse_error("No statement found in decorated statement"))?;

    Ok(DecoratedStatement {
        pos,
        comments,
        annotations,
        statement,
    })
}

fn parse_comment(pair: pest::iterators::Pair<Rule>) -> ParseResult<Comment> {
    let pos = parse_position(&pair);
    let text = pair.as_str().to_string();

    Ok(Comment {
        pos,
        text,
        nested_comments: Vec::new(), // For simplicity, no nested comments for now
    })
}

fn parse_annotation(pair: pest::iterators::Pair<Rule>) -> ParseResult<Annotation> {
    let pos = parse_position(&pair);
    let mut name = String::new();
    let mut arguments = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::tag => {
                name = inner_pair.as_str().to_string();
            }
            Rule::expression => {
                arguments.push(parse_expression(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(Annotation {
        pos,
        name,
        arguments,
    })
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<StatementKind> {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::module_stmt => {
                return Ok(StatementKind::Module(parse_module_statement(inner_pair)?))
            }
            Rule::function_stmt => {
                return Ok(StatementKind::Function(parse_function_statement(
                    inner_pair,
                )?))
            }
            Rule::return_stmt => {
                return Ok(StatementKind::Return(parse_return_statement(inner_pair)?))
            }
            Rule::let_stmt => return Ok(StatementKind::Let(parse_let_statement(inner_pair)?)),
            Rule::break_stmt => {
                return Ok(StatementKind::Break(parse_break_statement(inner_pair)?))
            }
            Rule::expression_stmt => {
                return Ok(StatementKind::Expression(parse_expression_statement(
                    inner_pair,
                )?))
            }
            _ => {}
        }
    }
    Err(parse_error("Unknown statement type"))
}

fn parse_module_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<Module> {
    let pos = parse_position(&pair);
    let mut name = String::new();
    let mut block = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => {
                name = inner_pair.as_str().to_string();
            }
            Rule::block => {
                block = Some(parse_block(inner_pair)?);
            }
            _ => {}
        }
    }

    let block = block.ok_or_else(|| parse_error("Module missing block"))?;

    Ok(Module {
        pos,
        name,
        body: block,
    })
}

fn parse_function_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<FunctionStatement> {
    let pos = parse_position(&pair);
    let mut is_static = false;
    let mut block = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::static_kw => {
                is_static = true;
            }
            Rule::block => {
                block = Some(parse_block(inner_pair)?);
            }
            _ => {}
        }
    }

    let block = block.ok_or_else(|| parse_error("Function missing block"))?;

    Ok(FunctionStatement {
        pos,
        is_static,
        block,
    })
}

fn parse_return_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<ReturnStatement> {
    let pos = parse_position(&pair);
    let mut label = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label => {
                label = Some(inner_pair.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok(ReturnStatement { pos, label })
}

fn parse_let_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<LetStatement> {
    let pos = parse_position(&pair);
    let mut is_static = false;
    let mut var_type = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::static_kw => {
                is_static = true;
            }
            Rule::atom => {
                var_type = Some(parse_atom(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(LetStatement {
        pos,
        is_static,
        var_type,
    })
}

fn parse_break_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<BreakStatement> {
    let pos = parse_position(&pair);
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                value = Some(parse_expression(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(BreakStatement { pos, value })
}

fn parse_expression_statement(
    pair: pest::iterators::Pair<Rule>,
) -> ParseResult<ExpressionStatement> {
    let pos = parse_position(&pair);
    let mut expression = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::expression => {
                expression = Some(parse_expression(inner_pair)?);
            }
            _ => {}
        }
    }

    let expression =
        expression.ok_or_else(|| parse_error("Expression statement missing expression"))?;

    Ok(ExpressionStatement { pos, expression })
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> ParseResult<Block> {
    let pos = parse_position(&pair);
    let mut label = None;
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label => {
                label = Some(inner_pair.as_str().to_string());
            }
            Rule::decorated_statement => {
                statements.push(parse_decorated_statement(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(Block {
        pos,
        label,
        statements,
    })
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::if_expression => return parse_if_expression(inner_pair),
            Rule::case_expression => return parse_case_expression(inner_pair),
            Rule::lambda => return parse_lambda(inner_pair),
            Rule::block_expression => return parse_block_expression(inner_pair),
            Rule::type_expression => return parse_type_expression(inner_pair),
            Rule::operation => return parse_operation(inner_pair),
            _ => {}
        }
    }

    // Fallback: try to parse as operation
    parse_operation(pair)
}

fn parse_if_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);
    let branches = Vec::new(); // Simplified for now

    Ok(ExpressionKind::If(IfExpression { pos, branches }))
}

fn parse_case_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);
    let branches = Vec::new(); // Simplified for now

    Ok(ExpressionKind::Case(CaseExpression { pos, branches }))
}

fn parse_lambda(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);
    let parameters = Vec::new(); // Simplified for now
    let return_type = None;
    let body = Block {
        pos: pos.clone(),
        label: None,
        statements: Vec::new(),
    };

    Ok(ExpressionKind::Lambda(Lambda {
        pos,
        parameters,
        return_type,
        body,
    }))
}

fn parse_block_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);
    let block = Block {
        pos: pos.clone(),
        label: None,
        statements: Vec::new(),
    };

    Ok(ExpressionKind::Block(BlockExpression { pos, block }))
}

fn parse_type_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);
    let type_expr = AtomKind::Binding(Binding {
        pos: pos.clone(),
        name: "placeholder".to_string(),
    });

    Ok(ExpressionKind::Type(TypeExpression { pos, type_expr }))
}

fn parse_operation(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);

    // For now, return a simple operand expression
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::operand {
            return parse_operand(inner_pair);
        }
    }

    // Fallback: create empty operation
    Ok(ExpressionKind::Operation(Operation {
        pos,
        operands: Vec::new(),
    }))
}

fn parse_operand(pair: pest::iterators::Pair<Rule>) -> ParseResult<ExpressionKind> {
    let pos = parse_position(&pair);
    let mut atom = None;
    let mut postfix = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::atom => {
                atom = Some(parse_atom(inner_pair)?);
            }
            Rule::postfix => {
                postfix.push(parse_postfix(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(ExpressionKind::Operand(Operand { pos, atom, postfix }))
}

fn parse_atom(pair: pest::iterators::Pair<Rule>) -> ParseResult<AtomKind> {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::grouped_expression => return parse_grouped_expression(inner_pair),
            Rule::atomic_type => return parse_atomic_type(inner_pair),
            Rule::atomic_value => return parse_atomic_value(inner_pair),
            Rule::binding => return parse_binding(inner_pair),
            _ => {}
        }
    }

    // Fallback: parse as binding
    let pos = parse_position(&pair);
    Ok(AtomKind::Binding(Binding {
        pos,
        name: pair.as_str().to_string(),
    }))
}

fn parse_grouped_expression(pair: pest::iterators::Pair<Rule>) -> ParseResult<AtomKind> {
    let pos = parse_position(&pair);
    let mut expression = None;

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::expression {
            expression = Some(Box::new(parse_expression(inner_pair)?));
        }
    }

    Ok(AtomKind::GroupedExpression(GroupedExpression {
        pos,
        expression,
    }))
}

fn parse_atomic_type(pair: pest::iterators::Pair<Rule>) -> ParseResult<AtomKind> {
    let pos = parse_position(&pair);
    let type_expr = AtomKind::Binding(Binding {
        pos: pos.clone(),
        name: "type_placeholder".to_string(),
    });

    Ok(AtomKind::AtomicType(AtomicType { pos, type_expr }))
}

fn parse_atomic_value(pair: pest::iterators::Pair<Rule>) -> ParseResult<AtomKind> {
    let pos = parse_position(&pair);
    let mut value = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::integer => {
                let int_value = inner_pair
                    .as_str()
                    .parse::<u64>()
                    .map_err(|_| parse_error("Invalid integer"))?;
                value = Some(ValueKind::Integer(Integer {
                    pos: parse_position(&inner_pair),
                    value: int_value,
                }));
            }
            Rule::string => {
                value = Some(ValueKind::String(StringLiteral {
                    pos: parse_position(&inner_pair),
                    fragments: Vec::new(), // Simplified for now
                }));
            }
            _ => {}
        }
    }

    Ok(AtomKind::AtomicValue(AtomicValue { pos, value }))
}

fn parse_binding(pair: pest::iterators::Pair<Rule>) -> ParseResult<AtomKind> {
    let pos = parse_position(&pair);
    let name = pair.as_str().to_string();

    Ok(AtomKind::Binding(Binding { pos, name }))
}

fn parse_postfix(pair: pest::iterators::Pair<Rule>) -> ParseResult<Postfix> {
    let pos = parse_position(&pair);

    // For now, return a simple call postfix
    Ok(Postfix::Call {
        pos,
        arguments: Vec::new(),
    })
}

// Simple test function
pub fn parse_simple_expression(input: &str) -> ParseResult<ExpressionKind> {
    // For testing, just return a simple integer operand
    Ok(ExpressionKind::Operand(Operand {
        pos: Position {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: input.len(),
        },
        atom: Some(AtomKind::AtomicValue(AtomicValue {
            pos: Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: input.len(),
            },
            value: Some(ValueKind::Integer(Integer {
                pos: Position {
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: input.len(),
                },
                value: input.parse().unwrap_or(0),
            })),
        })),
        postfix: Vec::new(),
    }))
}
