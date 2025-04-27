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
	Pos        lexer.Position       `json:"-" parser:""`
	Statements []DecoratedStatement `parser:"(@@ (EOL @@)*)? EOF"`
}

func (n Root) Position() lexer.Position { return n.Pos }
func (n Root) Children() []SyntaxNode {
	children := []SyntaxNode{}
	for _, stmt := range n.Statements {
		children = append(children, stmt)
	}
	return children
}

// Block represents a block of statements in the AST.
type Block struct {
	Pos        lexer.Position
	Label      string               `parser:"@Label?"`
	Statements []DecoratedStatement `parser:"'{' (@@ (EOL @@)*)? '}'"`
}

func (n Block) Position() lexer.Position { return n.Pos }
func (n Block) Children() []SyntaxNode {
	children := []SyntaxNode{}
	for _, stmt := range n.Statements {
		children = append(children, stmt)
	}
	return children
}

// DecoratedStatement represents a statement with comments and annotations.
type DecoratedStatement struct {
	Pos         lexer.Position `json:"-" parser:""`
	Comments    []Comment      `parser:"@@*"`
	Annotations []Annotation   `parser:"@@*"`
	Statement   Statement      `parser:"@@"`
}

func (n DecoratedStatement) Position() lexer.Position { return n.Pos }
func (n DecoratedStatement) Children() []SyntaxNode {
	children := []SyntaxNode{}
	for _, comment := range n.Comments {
		children = append(children, comment)
	}
	for _, annotation := range n.Annotations {
		children = append(children, annotation)
	}
	if n.Statement != nil {
		children = append(children, n.Statement.(SyntaxNode))
	}
	return children
}

// Comment represents a comment in the AST.
type Comment struct {
	Pos            lexer.Position `json:"-" parser:""`
	Text           string         `parser:"@(LineComment | BlockCommentContents)"`
	NestedComments []Comment      `parser:"| BlockCommentStart @@* BlockCommentEnd"`
}

func (n Comment) Position() lexer.Position { return n.Pos }
func (n Comment) Children() []SyntaxNode {
	children := []SyntaxNode{}
	for _, nestedComment := range n.NestedComments {
		children = append(children, nestedComment)
	}
	return children
}

// Annotation represents an annotation in the AST.
type Annotation struct {
	Pos       lexer.Position `json:"-" parser:""`
	Name      string         `parser:"@Tag"`
	Arguments []Expression   `parser:"('(' (@@ (',' @@)* ','?)? ')')?"`
}

func (n Annotation) Position() lexer.Position { return n.Pos }
func (n Annotation) Children() []SyntaxNode {
	children := []SyntaxNode{}
	for _, argument := range n.Arguments {
		if exprNode, ok := argument.(SyntaxNode); ok {
			children = append(children, exprNode)
		}
	}
	return children
}
