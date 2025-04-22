package ast

import (
	"fmt"
	"regexp"
	"strconv"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
)

type Type interface {
	is_type()
}

// PointerType represents a pointer type in the AST.
type PointerType struct {
	Pos       lexer.Position
	IsOwned   bool `parser:"'*' (@'own'"`
	IsRaw     bool `parser:"| @'raw'"`
	IsMutable bool `parser:"| @'mut')?"`
	Target    Atom `parser:"@@"`
}

func (PointerType) is_type() {}

func (n *PointerType) Position() lexer.Position { return n.Pos }
func (n *PointerType) Children() []SyntaxNode {
	if n.Target != nil {
		if targetNode, ok := n.Target.(SyntaxNode); ok {
			return []SyntaxNode{targetNode}
		}
	}
	return nil
}

// ArrayType represents an array type in the AST.
type ArrayType struct {
	Pos       lexer.Position
	Dimension *uint64 `parser:"'[' @Integer? ']'"`
	Element   Atom    `parser:"@@"`
}

func (ArrayType) is_type() {}

func (n *ArrayType) Position() lexer.Position { return n.Pos }
func (n *ArrayType) Children() []SyntaxNode {
	if n.Element != nil {
		if elementNode, ok := n.Element.(SyntaxNode); ok {
			return []SyntaxNode{elementNode}
		}
	}
	return nil
}

// IntegerType represents an integer type (signed or unsigned) with a specific bit size.
type IntegerType struct {
	Pos     lexer.Position
	Signed  bool
	BitSize uint64
}

func (IntegerType) is_type() {}

const (
	IntegerWidthBase    = 10
	IntegerWidthBitSize = 16
	IntegerWidthMin     = 1
	IntegerWidthMax     = (1 << IntegerWidthBitSize) - 1
	intSignedPrefix     = "i"
)

var (
	intWidthRange = fmt.Sprintf("[%d, %d]", IntegerWidthMin, IntegerWidthMax)
	intTypeRegex  = regexp.MustCompile(`^([iu])(\d+)`)
)

func (it *IntegerType) Parse(lex *lexer.PeekingLexer) error {
	token := lex.Peek()
	matches := intTypeRegex.FindStringSubmatch(token.Value)
	if matches == nil {
		return participle.NextMatch
	}

	sign, sizeStr := matches[1], matches[2]
	bitSize, err := strconv.ParseUint(sizeStr, IntegerWidthBase, IntegerWidthBitSize)
	if err != nil || bitSize == 0 {
		return participle.Wrapf(token.Pos, err, "Integer width of '%s' is out of range: %s", token.Value, intWidthRange)
	}

	lex.Next()
	it.Pos = token.Pos
	it.Signed = (sign == intSignedPrefix)
	it.BitSize = bitSize
	return nil
}

func (n *IntegerType) Position() lexer.Position { return n.Pos }
func (n *IntegerType) Children() []SyntaxNode {
	return nil
}

// FunctionType represents a function type in the AST.
type FunctionType struct {
	Pos        lexer.Position
	Name       string              `parser:"'fun' @Ident?"`
	Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
	Return     Atom                `parser:"(':' @@)?"`
}

func (FunctionType) is_type() {}

func (n *FunctionType) Position() lexer.Position { return n.Pos }
func (n *FunctionType) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Parameters)+1)
	for i := range n.Parameters {
		children = append(children, &n.Parameters[i])
	}
	if n.Return != nil {
		if returnNode, ok := n.Return.(SyntaxNode); ok {
			children = append(children, returnNode)
		}
	}
	return children
}
