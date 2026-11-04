//! Printer that converts AST back to source code

use crate::ast::*;

#[cfg(test)]
#[path = "printer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "printer_additional_tests.rs"]
mod additional_tests;

/// Printer that builds source code from AST
pub struct PrinterVisitor {
    pub output: String,
    indent_level: usize,
    indent_size: usize,
}

impl Default for PrinterVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PrinterVisitor {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size: 2,
        }
    }

    /// Get the generated source code
    pub fn get_output(self) -> String {
        self.output
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    fn write_indent(&mut self) {
        for _ in 0..(self.indent_level * self.indent_size) {
            self.output.push(' ');
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.write(s);
        self.output.push('\n');
    }

    fn write_indented(&mut self, s: &str) {
        self.write_indent();
        self.write(s);
    }

    /// Print root node
    pub fn print_root(&mut self, root: &Root) {
        for stmt in &root.statements {
            self.print_statement(stmt);
        }
    }

    /// Print a statement
    pub fn print_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Module(m) => self.print_module(m),
            Statement::Function(f) => self.print_function(f),
            Statement::Let(l) => self.print_let(l),
            Statement::Return(r) => self.print_return(r),
            Statement::Expression(e) => {
                self.write_indent();
                self.print_expression(e);
                self.writeln("");
            }
            Statement::While(w) => self.print_while(w),
            Statement::For(f) => self.print_for(f),
            Statement::Break(b) => self.print_break(b),
            Statement::Block(b) => {
                self.write_indent();
                self.print_block(b);
                self.writeln("");
            }
            Statement::Static(s) => self.print_static(s),
        }
    }

    /// Print a module
    pub fn print_module(&mut self, module: &Module) {
        self.write_indented(&format!("module {} {{\n", module.name));
        self.indent();
        for stmt in &module.statements {
            self.print_statement(stmt);
        }
        self.dedent();
        self.write_indented("}\n");
    }

    /// Print a function
    pub fn print_function(&mut self, func: &Function) {
        self.write_indent();
        if func.is_static {
            self.write("static ");
        }
        self.write(&format!("fn {}(", func.name));

        self.print_parameters(&func.parameters);

        self.write(")");

        if let Some(ret_type) = &func.return_type {
            self.write(": ");
            self.print_expression(ret_type);
        }

        if let Some(body) = &func.body {
            self.write(" ");
            self.print_block(body);
            self.writeln("");
        } else {
            self.writeln(";");
        }
    }

    fn print_parameters(&mut self, parameters: &[Parameter]) {
        for (i, param) in parameters.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&format!("{}: ", param.name));
            self.print_expression(&param.type_expr);
        }
    }

    /// Print a let statement
    pub fn print_let(&mut self, let_stmt: &Let) {
        self.write_indent();
        if let_stmt.is_static {
            self.write("static ");
        }
        self.write("let ");
        if let_stmt.is_mutable {
            self.write("mut ");
        }
        self.print_pattern(&let_stmt.pattern);

        if let Some(type_expr) = &let_stmt.type_expr {
            self.write(": ");
            self.print_expression(type_expr);
        }

        if let Some(value) = &let_stmt.value {
            self.write(" = ");
            self.print_expression(value);
        }

        self.writeln(";");
    }

    /// Print a pattern
    pub fn print_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(i) => self.write(&i.name),
            Pattern::Tuple(t) => {
                self.write("(");
                for (i, p) in t.patterns.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.print_pattern(p);
                }
                self.write(")");
            }
            Pattern::Struct(s) => {
                self.write(&format!("{} {{ ", s.name));
                for (i, (name, p)) in s.fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&format!("{}: ", name));
                    self.print_pattern(p);
                }
                self.write(" }");
            }
        }
    }

    /// Print a return statement
    pub fn print_return(&mut self, ret: &Return) {
        self.write_indent();
        self.write("return");
        if let Some(label) = &ret.label {
            self.write(&format!(" '{}", label));
        }
        if let Some(value) = &ret.value {
            self.write(" ");
            self.print_expression(value);
        }
        self.writeln(";");
    }

    /// Print a while loop
    pub fn print_while(&mut self, while_stmt: &While) {
        self.write_indent();
        if let Some(label) = &while_stmt.label {
            self.write(&format!("'{}: ", label));
        }
        self.write("while ");
        self.print_expression(&while_stmt.condition);
        self.write(" ");
        self.print_block(&while_stmt.body);
        self.writeln("");
    }

    /// Print a for loop
    pub fn print_for(&mut self, for_stmt: &For) {
        self.write_indent();
        if let Some(label) = &for_stmt.label {
            self.write(&format!("'{}: ", label));
        }
        self.write("for (");

        if let Some(init) = &for_stmt.init {
            if let Statement::Let(l) = init.as_ref() {
                if l.is_mutable {
                    self.write("mut ");
                }
                self.write("let ");
                self.print_pattern(&l.pattern);
                if let Some(type_expr) = &l.type_expr {
                    self.write(": ");
                    self.print_expression(type_expr);
                }
                if let Some(value) = &l.value {
                    self.write(" = ");
                    self.print_expression(value);
                }
            }
        }
        self.write("; ");

        if let Some(condition) = &for_stmt.condition {
            self.print_expression(condition);
        }
        self.write("; ");

        if let Some(update) = &for_stmt.update {
            self.print_expression(update);
        }

        self.write(") ");
        self.print_block(&for_stmt.body);
        self.writeln("");
    }

    /// Print a break statement
    pub fn print_break(&mut self, break_stmt: &Break) {
        self.write_indent();
        self.write("break");
        if let Some(label) = &break_stmt.label {
            self.write(&format!(" '{}", label));
        }
        if let Some(value) = &break_stmt.value {
            self.write(" ");
            self.print_expression(value);
        }
        self.writeln(";");
    }

    /// Print a block
    pub fn print_block(&mut self, block: &Block) {
        if let Some(label) = &block.label {
            self.write(&format!("'{}: ", label));
        }
        self.writeln("{");
        self.indent();
        for stmt in &block.statements {
            self.print_statement(stmt);
        }
        self.dedent();
        self.write_indent();
        self.write("}");
    }

    /// Print a static statement
    pub fn print_static(&mut self, static_stmt: &Static) {
        self.write_indented("static {\n");
        self.indent();
        for stmt in &static_stmt.statements {
            self.print_statement(stmt);
        }
        self.dedent();
        self.write_indented("}\n");
    }

    /// Print an expression
    pub fn print_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::If(i) => self.print_if(i),
            Expression::Case(c) => self.print_case(c),
            Expression::Lambda(l) => self.print_lambda(l),
            Expression::Operation(o) => self.print_operation(o),
            Expression::Call(c) => self.print_call(c),
            Expression::Index(i) => self.print_index(i),
            Expression::Field(f) => self.print_field(f),
            Expression::Literal(l) => self.print_literal(l),
            Expression::Identifier(i) => self.write(&i.name),
            Expression::Block(b) => {
                self.print_block(b);
            }
            Expression::Grouped(e) => {
                self.write("(");
                self.print_expression(e);
                self.write(")");
            }
        }
    }

    /// Print an if expression
    pub fn print_if(&mut self, if_expr: &If) {
        self.write("if ");

        // First branch
        if let Some(branch) = if_expr.branches.first() {
            self.print_expression(&branch.condition);
            self.write(" ");

            // Handle block body
            self.print_body(&branch.body);
        }

        // Remaining branches
        for branch in &if_expr.branches[1..] {
            self.write(" else if ");
            self.print_expression(&branch.condition);
            self.write(" ");

            self.print_body(&branch.body);
        }

        // Else branch
        if let Some(else_branch) = &if_expr.else_branch {
            self.write(" else ");
            match else_branch.as_ref() {
                Expression::Block(b) => {
                    self.print_block(b);
                }
                Expression::If(nested_if) => {
                    self.print_if(nested_if);
                }
                _ => {
                    self.writeln("{");
                    self.indent();
                    self.write_indent();
                    self.print_expression(else_branch);
                    self.writeln("");
                    self.dedent();
                    self.write_indent();
                    self.write("}");
                }
            }
        }
    }

    fn print_body(&mut self, body: &Expression) {
        match body {
            Expression::Block(b) => {
                self.print_block(b);
            }
            _ => {
                self.writeln("{");
                self.indent();
                self.write_indent();
                self.print_expression(body);
                self.writeln("");
                self.dedent();
                self.write_indent();
                self.write("}");
            }
        }
    }

    /// Print a case expression
    pub fn print_case(&mut self, case: &Case) {
        self.write("case ");
        self.print_expression(&case.value);
        self.writeln(" {");
        self.indent();

        for branch in &case.branches {
            self.write_indent();
            self.print_pattern(&branch.pattern);
            if let Some(guard) = &branch.guard {
                self.write(" if ");
                self.print_expression(guard);
            }
            self.write(" => ");
            self.print_expression(&branch.body);
            self.writeln(",");
        }

        self.dedent();
        self.write_indent();
        self.write("}");
    }

    /// Print a lambda
    pub fn print_lambda(&mut self, lambda: &Lambda) {
        self.write("\\(");
        self.print_parameters(&lambda.parameters);
        self.write(")");

        if let Some(ret_type) = &lambda.return_type {
            self.write(": ");
            self.print_expression(ret_type);
        }

        self.write(" => ");
        self.print_expression(&lambda.body);
    }

    /// Print an operation (in parse order)
    pub fn print_operation(&mut self, operation: &Operation) {
        let mut first = true;
        for element in &operation.elements {
            match element {
                OperationElement::Operand(expr) => {
                    if !first {
                        self.write(" ");
                    }
                    self.print_expression(expr);
                    first = false;
                }
                OperationElement::Operator(op) => {
                    self.write(" ");
                    self.write(op);
                }
            }
        }
    }

    /// Print a function call
    pub fn print_call(&mut self, call: &Call) {
        self.print_expression(&call.function);
        self.write("(");
        for (i, arg) in call.arguments.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.print_expression(arg);
        }
        self.write(")");
    }

    /// Print an index operation
    pub fn print_index(&mut self, index: &Index) {
        self.print_expression(&index.object);
        self.write("[");
        self.print_expression(&index.index);
        self.write("]");
    }

    /// Print a field access
    pub fn print_field(&mut self, field: &Field) {
        self.print_expression(&field.object);
        self.write(&format!(".{}", field.field));
    }

    /// Print a literal
    pub fn print_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Integer(i) => self.write(&i.value.to_string()),
            Literal::Float(f) => self.write(&f.value.to_string()),
            Literal::String(s) => self.print_string_literal(s),
            Literal::Boolean(b) => self.write(if b.value { "true" } else { "false" }),
            Literal::Array(a) => self.print_array_literal(a),
            Literal::Null(_) => self.write("null"),
        }
    }

    /// Print a string literal
    pub fn print_string_literal(&mut self, string: &StringLiteral) {
        self.write("\"");
        for fragment in &string.fragments {
            match fragment {
                StringFragment::Text(text) => {
                    for ch in text.chars() {
                        match ch {
                            '"' => self.write("\\\""),
                            '\\' => self.write("\\\\"),
                            '\n' => self.write("\\n"),
                            '\r' => self.write("\\r"),
                            '\t' => self.write("\\t"),
                            _ => self.output.push(ch),
                        }
                    }
                }
                StringFragment::Interpolation(expr) => {
                    self.write("${");
                    self.print_expression(expr);
                    self.write("}");
                }
            }
        }
        self.write("\"");
    }

    /// Print an array literal
    pub fn print_array_literal(&mut self, array: &ArrayLiteral) {
        self.write("[");
        for (i, element) in array.elements.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.print_expression(element);
        }
        self.write("]");
    }
}

// Implementation of Visitor trait for compatibility
impl Visitor for PrinterVisitor {
    // Empty implementations since we don't use the visitor pattern for printing
}
