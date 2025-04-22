package ast

import (
	"github.com/alecthomas/participle/v2/lexer"
)

type Statement interface{ statement() }

// Module represents a module declaration in the AST.
type Module struct {
	Pos  lexer.Position
	Name string `parser:"'module' @Ident"`
	Body Block  `parser:"@@"`
}

func (Module) statement() {}

func (n *Module) Position() lexer.Position { return n.Pos }
func (n *Module) Children() []SyntaxNode {
	return []SyntaxNode{&n.Body}
}

// FunctionStatement represents a function declaration in the AST.
type FunctionStatement struct {
	Pos        lexer.Position
	IsStatic   bool                `parser:"@'static'?"`
	Name       string              `parser:"'fun' @Ident"`
	Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
	Return     Atom                `parser:"(':' @@)?"`
	Block      Block               `parser:"@@"`
}

func (FunctionStatement) statement() {}

func (n *FunctionStatement) Position() lexer.Position { return n.Pos }
func (n *FunctionStatement) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Parameters)+1+1)
	for i := range n.Parameters {
		children = append(children, &n.Parameters[i])
	}
	if n.Return != nil {
		if returnNode, ok := n.Return.(SyntaxNode); ok {
			children = append(children, returnNode)
		}
	}
	children = append(children, &n.Block)
	return children
}

// FunctionParameter represents a parameter in a function declaration.
type FunctionParameter struct {
	Pos  lexer.Position
	Name string `parser:"@Ident ':'"`
	Type Atom   `parser:"@@"`
}

func (n *FunctionParameter) Position() lexer.Position { return n.Pos }
func (n *FunctionParameter) Children() []SyntaxNode {
	if n.Type != nil {
		if typeNode, ok := n.Type.(SyntaxNode); ok {
			return []SyntaxNode{typeNode}
		}
	}
	return nil
}

// ReturnStatement represents a return statement in the AST.
type ReturnStatement struct {
	Pos   lexer.Position
	Label string     `parser:"'return' @Label?"`
	Value Expression `parser:"@@?"`
}

func (ReturnStatement) statement() {}

func (n *ReturnStatement) Position() lexer.Position { return n.Pos }
func (n *ReturnStatement) Children() []SyntaxNode {
	if n.Value != nil {
		if valueNode, ok := n.Value.(SyntaxNode); ok {
			return []SyntaxNode{valueNode}
		}
	}
	return nil
}

// LetStatement represents a variable declaration in the AST.
type LetStatement struct {
	Pos       lexer.Position
	IsStatic  bool       `parser:"@'static'?"`
	IsMutable bool       `parser:"'let' @'mut'?"`
	Name      string     `parser:"@Ident"`
	Type      Atom       `parser:"(':' @@)?"`
	Value     Expression `parser:"('=' @@)?"`
}

func (LetStatement) statement() {}

func (n *LetStatement) Position() lexer.Position { return n.Pos }
func (n *LetStatement) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, 2)
	if n.Type != nil {
		if typeNode, ok := n.Type.(SyntaxNode); ok {
			children = append(children, typeNode)
		}
	}
	if n.Value != nil {
		if valueNode, ok := n.Value.(SyntaxNode); ok {
			children = append(children, valueNode)
		}
	}
	return children
}

// BreakStatement represents a break statement in the AST.
type BreakStatement struct {
	Pos   lexer.Position
	Label string     `parser:"'break' @Label?"`
	Value Expression `parser:"@@?"`
}

func (BreakStatement) statement() {}

func (n *BreakStatement) Position() lexer.Position { return n.Pos }
func (n *BreakStatement) Children() []SyntaxNode {
	if n.Value != nil {
		if valueNode, ok := n.Value.(SyntaxNode); ok {
			return []SyntaxNode{valueNode}
		}
	}
	return nil
}

// ExpressionStatement represents an expression used as a statement.
type ExpressionStatement struct {
	Pos        lexer.Position
	Expression Expression `parser:"@@"`
}

func (ExpressionStatement) statement() {}

func (n *ExpressionStatement) Position() lexer.Position { return n.Pos }
func (n *ExpressionStatement) Children() []SyntaxNode {
	if n.Expression != nil {
		if exprNode, ok := n.Expression.(SyntaxNode); ok {
			return []SyntaxNode{exprNode}
		}
	}
	return nil
}
