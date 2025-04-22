package ast

import (
	"github.com/alecthomas/participle/v2/lexer"
)

type Expression interface {
	expression()
}

// IfExpression represents a conditional expression with if/else branches.
type IfExpression struct {
	Pos     lexer.Position
	If      IfBranch   `parser:"@@"`
	ElseIfs []IfBranch `parser:"('else' @@)*"`
	Else    Expression `parser:"('else' @@)?"`
}

func (IfExpression) expression() {}

func (n *IfExpression) Position() lexer.Position { return n.Pos }
func (n *IfExpression) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, 1+len(n.ElseIfs)+1)
	children = append(children, &n.If)
	for i := range n.ElseIfs {
		children = append(children, &n.ElseIfs[i])
	}
	if n.Else != nil {
		if elseNode, ok := n.Else.(SyntaxNode); ok {
			children = append(children, elseNode)
		}
	}
	return children
}

// IfBranch represents a single condition-expression branch of an if statement.
type IfBranch struct {
	Pos       lexer.Position
	Condition Expression `parser:"'if' '(' @@ ')'"`
	Then      Expression `parser:"@@"`
}

func (n *IfBranch) Position() lexer.Position { return n.Pos }
func (n *IfBranch) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, 2)
	if n.Condition != nil {
		if condNode, ok := n.Condition.(SyntaxNode); ok {
			children = append(children, condNode)
		}
	}
	if n.Then != nil {
		if thenNode, ok := n.Then.(SyntaxNode); ok {
			children = append(children, thenNode)
		}
	}
	return children
}

// CaseExpression represents a case expression with multiple branches.
type CaseExpression struct {
	Pos      lexer.Position
	Branches []CaseBranch `parser:"'case' '{' @@+ '}'"`
}

func (CaseExpression) expression() {}

func (n *CaseExpression) Position() lexer.Position { return n.Pos }
func (n *CaseExpression) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Branches))
	for i := range n.Branches {
		children = append(children, &n.Branches[i])
	}
	return children
}

// CaseBranch represents a single branch in a case expression.
type CaseBranch struct {
	Pos  lexer.Position
	If   *IfBranch  `parser:"@@"`
	Else Expression `parser:"| 'else' @@"`
}

func (n *CaseBranch) Position() lexer.Position { return n.Pos }
func (n *CaseBranch) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, 2)
	if n.If != nil {
		children = append(children, n.If)
	}
	if n.Else != nil {
		if elseNode, ok := n.Else.(SyntaxNode); ok {
			children = append(children, elseNode)
		}
	}
	return children
}

// Lambda represents a function literal expression.
type Lambda struct {
	Pos        lexer.Position
	Parameters []FunctionParameter `parser:"'fun' '(' (@@ (',' @@)* ','?)? ')'"`
	Return     Atom                `parser:"(':' @@)?"`
	Body       Block               `parser:"@@"`
}

func (Lambda) expression() {}

func (n *Lambda) Position() lexer.Position { return n.Pos }
func (n *Lambda) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Parameters)+1+1)
	for i := range n.Parameters {
		children = append(children, &n.Parameters[i])
	}
	if n.Return != nil {
		if returnNode, ok := n.Return.(SyntaxNode); ok {
			children = append(children, returnNode)
		}
	}
	children = append(children, &n.Body)
	return children
}

// BlockExpression represents a block of statements used as an expression.
type BlockExpression struct {
	Pos      lexer.Position
	IsStatic bool  `parser:"@'static'?"`
	Block    Block `parser:"@@"`
}

func (BlockExpression) expression() {}

func (n *BlockExpression) Position() lexer.Position { return n.Pos }
func (n *BlockExpression) Children() []SyntaxNode {
	return []SyntaxNode{&n.Block}
}

// TypeExpression represents a type used as an expression.
type TypeExpression struct {
	Pos  lexer.Position
	Type Type `parser:"@@"`
}

func (TypeExpression) expression() {}

func (n *TypeExpression) Position() lexer.Position { return n.Pos }
func (n *TypeExpression) Children() []SyntaxNode {
	if n.Type != nil {
		if typeNode, ok := n.Type.(SyntaxNode); ok {
			return []SyntaxNode{typeNode}
		}
	}
	return nil
}

// Operation represents a sequence of operations on operands.
type Operation struct {
	Pos      lexer.Position
	Prefix   []string  `parser:"@Operator*"`
	Operands []Operand `parser:"@@+"`
}

func (Operation) expression() {}

func (n *Operation) Position() lexer.Position { return n.Pos }
func (n *Operation) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Operands))
	for i := range n.Operands {
		children = append(children, &n.Operands[i])
	}
	return children
}

// Operand represents a single operand in an operation.
type Operand struct {
	Atom    Atom      `parser:"@@"`
	Postfix []Postfix `parser:"@@*"`
}

func (n *Operand) Position() lexer.Position {
	if n.Atom != nil {
		if atomNode, ok := n.Atom.(SyntaxNode); ok {
			return atomNode.Position()
		}
	}
	return lexer.Position{}
}

func (n *Operand) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, 1+len(n.Postfix))
	if n.Atom != nil {
		if atomNode, ok := n.Atom.(SyntaxNode); ok {
			children = append(children, atomNode)
		}
	}
	for i := range n.Postfix {
		children = append(children, &n.Postfix[i])
	}
	return children
}

// Postfix represents a postfix operation on an operand.
type Postfix struct {
	Pos        lexer.Position
	Invocation []Expression `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
	Index      Expression   `parser:"| '[' @@ ']'"`
	Operator   string       `parser:"| @Operator"`
}

func (n *Postfix) Position() lexer.Position { return n.Pos }
func (n *Postfix) Children() []SyntaxNode {
	children := make([]SyntaxNode, 0, len(n.Invocation)+1)
	for i := range n.Invocation {
		if exprNode, ok := n.Invocation[i].(SyntaxNode); ok {
			children = append(children, exprNode)
		}
	}
	if n.Index != nil {
		if indexNode, ok := n.Index.(SyntaxNode); ok {
			children = append(children, indexNode)
		}
	}
	return children
}
