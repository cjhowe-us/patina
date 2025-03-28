package patina

import (
	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
)

var (
	lex = lexer.MustStateful(lexer.Rules{
		"Root": {
			{Name: "BracketLeft", Pattern: `[([{]`, Action: lexer.Push("Root")},
			{Name: "BracketRight", Pattern: `[)\]}]`, Action: lexer.Pop()},
			{Name: "UnaryOperator", Pattern: `[~!&*-]`, Action: nil},
			{Name: "BinaryOperator", Pattern: `[~!@#$%^&*\-+=:;<>?|,.\\]{2,}`, Action: nil},
			{Name: "Integer", Pattern: `\d+`, Action: nil},
			{Name: "StringStart", Pattern: `"`, Action: lexer.Push("String")},
			{Name: "Ident", Pattern: "[a-zA-Z_][a-zA-Z0-9_]*|`[^`\\s]*`", Action: nil},
			{Name: "Whitespace", Pattern: `[ \t\n\r]+`, Action: nil},
			{Name: "LineComment", Pattern: `(?://)[^\n]*\n?`, Action: nil},
			lexer.Include("DocCommentStart"),
			lexer.Include("BlockCommentStart"),
		},
		"DocCommentStart": {
			{Name: "DocCommentStart", Pattern: `/\*\*`, Action: lexer.Push("DocComment")},
		},
		"DocComment": {
			{Name: "DocCommentEnd", Pattern: `\*/`, Action: lexer.Pop()},
			lexer.Include("DocCommentStart"),
			{Name: "DocCommentContents", Pattern: `([^*/]|/[^*]|\*[^/])+`, Action: nil},
		},
		"BlockCommentStart": {
			{Name: "BlockCommentStart", Pattern: `/\*`, Action: lexer.Push("BlockComment")},
		},
		"BlockComment": {
			{Name: "BlockCommentEnd", Pattern: `\*/`, Action: lexer.Pop()},
			lexer.Include("BlockCommentStart"),
			{Name: "BlockCommentContents", Pattern: `([^*/]|/[^*]|\*[^/])+`, Action: nil},
		},
		"String": {
			{Name: "StringEnd", Pattern: `"`, Action: lexer.Pop()},
			{Name: "StringInterpolant", Pattern: `\\\(`, Action: lexer.Push("Root")},
			{Name: "StringFragment", Pattern: `[^"\\]+`, Action: nil},
		},
	})
	Parser = participle.MustBuild[Root](
		participle.Lexer(lex),
		participle.Elide("Whitespace", "BlockCommentStart", "BlockCommentContents", "BlockCommentEnd", "LineComment"),
		participle.UseLookahead(20),
		participle.Union[Statement](Module{}, Block{}, BindStatement{}, FunctionStatement{}, IfStatement{}, ExpressionStatement{}),
		participle.Union[Atom](Lambda{}, FunctionCall{}, StaticBlock{}, GroupedExpression{}, UnaryOperation{}, AtomValue{}, BindingAccess{}),
		participle.Union[TypeValue](IntegerType{}, PointerType{}, FunctionType{}, ArrayType{}, TypeExpression{}),
		participle.Union[Value](Integer{}, String{}, IntegerType{}, PointerType{}, FunctionType{}, ArrayType{}, TypeExpression{}),
	)
)

type (
	Root struct {
		Statements []DocumentedStatement `parser:"@@*"`
	}

	Statement           interface{ statement() }
	DocumentedStatement struct {
		Pos        lexer.Position
		DocComment string    `parser:"@(DocCommentStart DocCommentContents* DocCommentEnd)?"`
		Statement  Statement `parser:"@@"`
	}
	Block struct {
		Pos        lexer.Position
		Statements []DocumentedStatement `parser:"'{' @@* '}'"`
	}
	Module struct {
		Pos  lexer.Position
		Name string `parser:"'module' @Ident"`
		Body Block  `parser:"@@"`
	}
	BindStatement struct {
		Pos       lexer.Position
		IsStatic  bool        `parser:"@'static'?"`
		IsMutable bool        `parser:"'let' @'mut'?"`
		Name      string      `parser:"@Ident"`
		Type      *TypeValue  `parser:"(':' @@)?"`
		Expr      *Expression `parser:"('=' @@)?"`
	}
	FunctionStatement struct {
		Pos        lexer.Position
		Name       string              `parser:"'fun' @Ident"`
		Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
		Return     *TypeValue          `parser:"(':' @@)?"`
		Block      Block               `parser:"@@"`
	}
	IfStatement struct {
		Pos              lexer.Position
		Condition        Expression    `parser:"'if' @@"`
		ThenBlock        Block         `parser:"@@"`
		ElseIfStatements []IfStatement `parser:"('else' @@)*"`
		ElseBlock        *Block        `parser:"('else' @@)?"`
	}
	ExpressionStatement struct {
		Pos        lexer.Position
		Expression Expression `parser:"@@"`
	}

	Expression struct {
		Pos        lexer.Position
		Operand    Atom              `parser:"@@"`
		Operations []BinaryOperation `parser:"@@*"`
	}
	BinaryOperation struct {
		Operator string `parser:"@(UnaryOperator | BinaryOperator)"`
		Operand  Atom   `parser:"@@"`
	}

	Atom   interface{ atom() }
	Lambda struct {
		Pos        lexer.Position
		Parameters []FunctionParameter `parser:"'fun' '(' [ @@ (',' @@)* ','? ] ')'"`
		Return     *TypeValue          `parser:"[':' @@]"`
		Block      Block               `parser:"@@"`
	}
	FunctionCall struct {
		Pos       lexer.Position
		Target    Atom         `parser:"@@"`
		Arguments []Expression `parser:"'(' [ @@ (',' @@)* ','? ] ')'"`
	}
	StaticBlock struct {
		Pos  lexer.Position
		Body Block `parser:"'static' @@"`
	}
	GroupedExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'(' @@ ')'"`
	}
	UnaryOperation struct {
		Pos      lexer.Position
		Operator string `parser:"@UnaryOperator"`
		Operand  Atom   `parser:"@@"`
	}
	AtomValue struct {
		Pos   lexer.Position
		Value Value `parser:"@@"`
	}
	BindingAccess struct {
		Pos  lexer.Position
		Name string `parser:"@Ident"`
	}

	Value   interface{ value() }
	Integer struct {
		Pos   lexer.Position
		Value uint64 `parser:"@Integer"`
	}
	String struct {
		Pos       lexer.Position
		Fragments []StringFragment `parser:"StringStart @@* StringEnd"`
	}
	StringFragment struct {
		Pos           lexer.Position
		Text          string      `parser:"@StringFragment+"`
		Interpolation *Expression `parser:"| StringInterpolant @@ ')'"`
	}

	TypeValue interface {
		Value
		type_value()
	}
	IntegerType struct {
		Pos     lexer.Position
		Signed  bool   `parser:"(@'i' | 'u')"`
		BitSize uint64 `parser:"@Integer"`
	}
	PointerType struct {
		Pos        lexer.Position
		IsOwned    bool      `parser:"@'@'"`
		IsMutable  bool      `parser:"| '*' @'mut'?"`
		TargetType TypeValue `parser:"@@"`
	}
	ArrayType struct {
		Pos         lexer.Position
		Dimensions  []uint64  `parser:"'[' (@Integer (',' @Integer)* ','?)? ']'"`
		ElementType TypeValue `parser:"@@"`
	}
	FunctionType struct {
		Pos        lexer.Position
		Name       string              `parser:"'fun' @Ident?"`
		Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)*)? ')'"`
		Return     *TypeValue          `parser:"(':' @@)? (?! '{')"`
	}
	FunctionParameter struct {
		Pos  lexer.Position
		Name string    `parser:"@Ident"`
		Type TypeValue `parser:"':' @@"`
	}
	TypeExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'\\\\' '(' @@ ')'"`
	}
)

func (Module) statement()              {}
func (Block) statement()               {}
func (BindStatement) statement()       {}
func (FunctionStatement) statement()   {}
func (IfStatement) statement()         {}
func (ExpressionStatement) statement() {}

func (StaticBlock) atom()       {}
func (Lambda) atom()            {}
func (FunctionCall) atom()      {}
func (GroupedExpression) atom() {}
func (UnaryOperation) atom()    {}
func (AtomValue) atom()         {}
func (BindingAccess) atom()     {}

func (Integer) value() {}
func (String) value()  {}

func (IntegerType) type_value()    {}
func (PointerType) type_value()    {}
func (ArrayType) type_value()      {}
func (FunctionType) type_value()   {}
func (TypeExpression) type_value() {}
func (IntegerType) value()         {}
func (PointerType) value()         {}
func (ArrayType) value()           {}
func (FunctionType) value()        {}
func (TypeExpression) value()      {}
