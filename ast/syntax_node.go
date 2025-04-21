package ast

import (
	"github.com/alecthomas/participle/v2/lexer"
)

type SyntaxNode interface {
	Position() lexer.Position
	Children() []SyntaxNode
}

// Root represents the root of an AST tree.
type Root struct {
	Pos        lexer.Position
	Statements []DecoratedStatement `parser:"(@@ (EOL @@)*)? EOF"`
}

func (n *Root) Position() lexer.Position { return n.Pos }
func (n *Root) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Statements))
	for i := range n.Statements {
		children = append(children, &n.Statements[i])
	}
	return children
}

// Block represents a block of statements in the AST.
type Block struct {
	Pos        lexer.Position
	Label      string               `parser:"@Label?"`
	Statements []DecoratedStatement `parser:"'{' (@@ (EOL @@)*)? '}'"`
}

func (n *Block) Position() lexer.Position { return n.Pos }
func (n *Block) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Statements))
	for i := range n.Statements {
		children = append(children, &n.Statements[i])
	}
	return children
}

// DecoratedStatement represents a statement with comments and annotations.
type DecoratedStatement struct {
	Pos         lexer.Position
	Comments    *Comment     `parser:"@@*"`
	Annotations []Annotation `parser:"@@*"`
	Statement   Statement    `parser:"@@"`
}

func (n *DecoratedStatement) Position() lexer.Position { return n.Pos }
func (n *DecoratedStatement) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, 1+len(n.Annotations))
	if n.Comments != nil {
		children = append(children, n.Comments)
	}
	for i := range n.Annotations {
		children = append(children, &n.Annotations[i])
	}
	if n.Statement != nil {
		if stmtNode, ok := n.Statement.(SyntaxNode); ok {
			children = append(children, stmtNode)
		}
	}
	return children
}

// Comment represents a comment in the AST.
type Comment struct {
	Pos            lexer.Position
	Text           string    `parser:"@(LineComment | BlockCommentContents)"`
	NestedComments []Comment `parser:"| BlockCommentStart @@* BlockCommentEnd"`
}

func (n *Comment) Position() lexer.Position { return n.Pos }
func (n *Comment) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.NestedComments))
	for i := range n.NestedComments {
		children = append(children, &n.NestedComments[i])
	}
	return children
}

// Annotation represents an annotation in the AST.
type Annotation struct {
	Pos       lexer.Position
	Name      string       `parser:"@Tag"`
	Arguments []Expression `parser:"('(' (@@ (',' @@)* ','?)? ')')?"`
}

func (n *Annotation) Position() lexer.Position { return n.Pos }
func (n *Annotation) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Arguments))
	for i := range n.Arguments {
		if exprNode, ok := n.Arguments[i].(SyntaxNode); ok {
			children = append(children, exprNode)
		}
	}
	return children
}
