package ast

import (
	"github.com/alecthomas/participle/v2/lexer"
)

type Value interface{ value() }

// Integer represents an integer literal in the AST.
type Integer struct {
	Pos   lexer.Position
	Value uint64 `parser:"@Integer"`
}

func (Integer) value() {}

func (n *Integer) Position() lexer.Position { return n.Pos }
func (n *Integer) Children() []SyntaxNode {
	return nil
}

// String represents a string literal in the AST.
type String struct {
	Pos       lexer.Position
	Fragments []StringFragment `parser:"StringStart @@* StringEnd"`
}

func (String) value() {}

func (n *String) Position() lexer.Position { return n.Pos }
func (n *String) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Fragments))
	for i := range n.Fragments {
		children = append(children, &n.Fragments[i])
	}
	return children
}

// StringFragment represents a fragment of a string literal, which can be either text or an interpolation.
type StringFragment struct {
	Pos           lexer.Position
	Text          string     `parser:"@StringFragment+"`
	Interpolation Expression `parser:"| StringInterpolant @@ ')'"`
}

func (n *StringFragment) Position() lexer.Position { return n.Pos }
func (n *StringFragment) Children() []SyntaxNode {
	if n.Interpolation != nil {
		if interpNode, ok := n.Interpolation.(SyntaxNode); ok {
			return []SyntaxNode{interpNode}
		}
	}
	return nil
}
