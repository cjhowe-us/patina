package patina

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ParseNode is the interface implemented by all AST nodes
type ParseNode interface {
	_parseNode()
}

// Root AST node
type Root struct {
	Pos        lexer.Position
	Statements []DecoratedStatement `parser:"(@@ (EOL @@)*)? EOF"`
}

func (Root) _parseNode() {}

// Decoration types for AST nodes
type DecoratedStatement struct {
	Pos         lexer.Position
	Comments    *DocComment  `parser:"@@?"`
	Annotations []Annotation `parser:"@@*"`
	Statement   Statement    `parser:"@@"`
}

func (DecoratedStatement) _parseNode() {}

type DocComment struct {
	Pos       lexer.Position
	Fragments []DocCommentFragment `parser:"BlockCommentStart @@* BlockCommentEnd"`
}

func (DocComment) _parseNode() {}

type DocCommentFragment struct {
	Pos     lexer.Position
	Text    string      `parser:"@BlockCommentContents"`
	Comment *DocComment `parser:"| @@"`
}

func (DocCommentFragment) _parseNode() {}

type Annotation struct {
	Pos       lexer.Position
	Name      string       `parser:"@Tag"`
	Arguments []Expression `parser:"('(' (@@ (',' @@)* ','?)? ')')?"`
}

func (Annotation) _parseNode() {}

// Function parameter
type FunctionParameter struct {
	Pos  lexer.Position
	Name string `parser:"@Ident"`
	Type Atom   `parser:"':' @@"`
}

func (FunctionParameter) _parseNode() {}

// Statement interface and implementations
type Statement interface {
	_statement()
}

type Module struct {
	Pos  lexer.Position
	Name string `parser:"'module' @Ident"`
	Body Block  `parser:"@@"`
}

func (Module) _statement() {}
func (Module) _parseNode() {}

type FunctionStatement struct {
	Pos        lexer.Position
	IsStatic   bool                `parser:"@'static'?"`
	Name       string              `parser:"'fun' @Ident"`
	Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
	Return     Atom                `parser:"(':' @@)?"`
	Block      Block               `parser:"@@"`
}

func (FunctionStatement) _statement() {}
func (FunctionStatement) _parseNode() {}

type ReturnStatement struct {
	Pos   lexer.Position
	Label string     `parser:"'return' @Label?"`
	Value Expression `parser:"@@?"`
}

func (ReturnStatement) _statement() {}
func (ReturnStatement) _parseNode() {}

type LetStatement struct {
	Pos       lexer.Position
	IsStatic  bool       `parser:"@'static'?"`
	IsMutable bool       `parser:"'let' @'mut'?"`
	Name      string     `parser:"@Ident"`
	Type      Atom       `parser:"(':' @@)?"`
	Value     Expression `parser:"('=' @@)? (?= EOL | EOF)"`
}

func (LetStatement) _statement() {}
func (LetStatement) _parseNode() {}

type BreakStatement struct {
	Pos   lexer.Position
	Label string     `parser:"'break' @Label?"`
	Value Expression `parser:"@@?"`
}

func (BreakStatement) _statement() {}
func (BreakStatement) _parseNode() {}

type ExpressionStatement struct {
	Pos        lexer.Position
	Expression Expression `parser:"@@"`
}

func (ExpressionStatement) _statement() {}
func (ExpressionStatement) _parseNode() {}

// Expression interface and implementations
type Expression interface {
	_expression()
}

type Block struct {
	Pos        lexer.Position
	Label      string               `parser:"@Label?"`
	Statements []DecoratedStatement `parser:"'{' (@@ (EOL @@)*)? '}'"`
}

func (Block) _expression() {}
func (Block) _parseNode()  {}

type IfExpression struct {
	Pos     lexer.Position
	If      IfBranch   `parser:"@@"`
	ElseIfs []IfBranch `parser:"('else' @@)*"`
	Else    Expression `parser:"('else' @@)?"`
}

func (IfExpression) _expression() {}
func (IfExpression) _parseNode()  {}

type IfBranch struct {
	Pos       lexer.Position
	Condition Expression `parser:"'if' '(' @@ ')'"`
	Then      Expression `parser:"@@"`
}

func (IfBranch) _parseNode() {}

type CaseExpression struct {
	Pos      lexer.Position
	Branches []CaseBranch `parser:"'case' '{' @@+ '}'"`
}

func (CaseExpression) _expression() {}
func (CaseExpression) _parseNode()  {}

type CaseBranch struct {
	Pos  lexer.Position
	If   *IfBranch  `parser:"@@"`
	Else Expression `parser:"| 'else' @@"`
}

func (CaseBranch) _parseNode() {}

type Lambda struct {
	Pos        lexer.Position
	Parameters []FunctionParameter `parser:"'fun' '(' (@@ (',' @@)* ','?)? ')'"`
	Return     Atom                `parser:"(':' @@)?"`
	Body       Block               `parser:"@@"`
}

func (Lambda) _expression() {}
func (Lambda) _parseNode()  {}

type StaticExpression struct {
	Pos    lexer.Position
	Static Expression `parser:"'static' @@"`
}

func (StaticExpression) _expression() {}
func (StaticExpression) _parseNode()  {}

type TypeExpression struct {
	Pos  lexer.Position
	Type Type `parser:"@@"`
}

func (TypeExpression) _expression() {}
func (TypeExpression) _parseNode()  {}

type Operation struct {
	Pos      lexer.Position
	Prefix   []string  `parser:"@Operator*"`
	Operands []Operand `parser:"@@+"`
}

func (Operation) _expression() {}
func (Operation) _parseNode()  {}

type Operand struct {
	Atom    Atom      `parser:"@@"`
	Postfix []Postfix `parser:"@@*"`
}

func (Operand) _parseNode() {}

type Postfix struct {
	Pos        lexer.Position
	Invocation []Expression `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
	Index      Expression   `parser:"| '[' @@ ']'"`
	Operator   string       `parser:"| @Operator"`
}

func (Postfix) _parseNode() {}

// Atom interface and implementations
type Atom interface {
	_atom()
}

type GroupedExpression struct {
	Pos        lexer.Position
	Expression Expression `parser:"'(' @@ ')'"`
}

func (GroupedExpression) _atom()      {}
func (GroupedExpression) _parseNode() {}

type AtomicType struct {
	Pos  lexer.Position
	Type Type `parser:"@@"`
}

func (AtomicType) _atom()      {}
func (AtomicType) _parseNode() {}

type AtomicValue struct {
	Pos   lexer.Position
	Value Value `parser:"@@"`
}

func (AtomicValue) _atom()      {}
func (AtomicValue) _parseNode() {}

type Binding struct {
	Pos   lexer.Position
	Ident string `parser:"@Ident"`
}

func (Binding) _atom()      {}
func (Binding) _parseNode() {}

// Value interface and implementations
type Value interface {
	_value()
}

type Integer struct {
	Pos   lexer.Position
	Value uint64 `parser:"@Integer"`
}

func (Integer) _value()     {}
func (Integer) _parseNode() {}

type String struct {
	Pos       lexer.Position
	Fragments []StringFragment `parser:"StringStart @@* StringEnd"`
}

func (String) _value()     {}
func (String) _parseNode() {}

type StringFragment struct {
	Pos           lexer.Position
	Text          string     `parser:"@StringFragment+"`
	Interpolation Expression `parser:"| StringInterpolant @@ ')'"`
}

func (StringFragment) _parseNode() {}

// Type interface and implementations
type Type interface {
	_type()
}

type IntegerType struct {
	Pos     lexer.Position
	Signed  bool
	BitSize uint64
}

func (IntegerType) _type()      {}
func (IntegerType) _parseNode() {}

type PointerType struct {
	Pos       lexer.Position
	IsOwned   bool `parser:"'*' (@'own'"`
	IsRaw     bool `parser:"| @'raw'"`
	IsMutable bool `parser:"| @'mut')?"`
	Target    Atom `parser:"@@"`
}

func (PointerType) _type()      {}
func (PointerType) _parseNode() {}

type ArrayType struct {
	Pos       lexer.Position
	Dimension *uint64 `parser:"'[' @Integer? ']'"`
	Element   Atom    `parser:"@@"`
}

func (ArrayType) _type()      {}
func (ArrayType) _parseNode() {}

type FunctionType struct {
	Pos        lexer.Position
	Name       string              `parser:"'fun' @Ident?"`
	Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
	Return     Atom                `parser:"(':' @@)? (?! '{')"`
}

func (FunctionType) _type()      {}
func (FunctionType) _parseNode() {}
