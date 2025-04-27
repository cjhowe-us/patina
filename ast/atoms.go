package ast

import (
	"github.com/alecthomas/participle/v2/lexer"
)

type Atom interface{ atom() }

// GroupedExpression represents a parenthesized expression in the AST.
type GroupedExpression struct {
	Pos        lexer.Position `json:"-" parser:""`
	Expression Expression     `parser:"'(' @@ ')'"`
}

func (GroupedExpression) atom() {}

func (n GroupedExpression) Position() lexer.Position { return n.Pos }
func (n GroupedExpression) Children() []SyntaxNode {
	if n.Expression != nil {
		if exprNode, ok := n.Expression.(SyntaxNode); ok {
			return []SyntaxNode{exprNode}
		}
	}
	return nil
}

// AtomicType represents a type used as an atomic expression.
type AtomicType struct {
	Pos  lexer.Position `json:"-" parser:""`
	Type Type           `parser:"@@"`
}

func (AtomicType) atom() {}

func (n AtomicType) Position() lexer.Position { return n.Pos }
func (n AtomicType) Children() []SyntaxNode {
	if n.Type != nil {
		if typeNode, ok := n.Type.(SyntaxNode); ok {
			return []SyntaxNode{typeNode}
		}
	}
	return nil
}

// AtomicValue represents a value used as an atomic expression.
type AtomicValue struct {
	Pos   lexer.Position `json:"-" parser:""`
	Value Value          `parser:"@@"`
}

func (AtomicValue) atom() {}

func (n AtomicValue) Position() lexer.Position { return n.Pos }
func (n AtomicValue) Children() []SyntaxNode {
	if n.Value != nil {
		if valueNode, ok := n.Value.(SyntaxNode); ok {
			return []SyntaxNode{valueNode}
		}
	}
	return nil
}

// Binding represents an identifier binding in the AST.
type Binding struct {
	Pos   lexer.Position `json:"-" parser:""`
	Ident string         `parser:"@Ident"`
}

func (Binding) atom() {}

func (n Binding) Position() lexer.Position { return n.Pos }
func (n Binding) Children() []SyntaxNode {
	return nil
}
