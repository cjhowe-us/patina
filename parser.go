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
			{Name: "Operator", Pattern: `[~!@#$%^&*-+=:;<>?|.\\]{1,5}`, Action: nil},
			{Name: "Integer", Pattern: `\d`, Action: nil},
			{Name: "StringStart", Pattern: `"`, Action: lexer.Push("String")},
			{Name: "Ident", Pattern: "[a-zA-Z_][a-zA-Z0-9_]*|`[^`\\s]*`", Action: nil},
			{Name: "Whitespace", Pattern: `\s+`, Action: nil},
			{Name: "LineComment", Pattern: `(?://)[^\n]*\n?`, Action: nil},
			lexer.Include("DocCommentStart"),
			lexer.Include("BlockCommentStart"),
		},
		"DocCommentStart": {
			{Name: "DocCommentStart", Pattern: `/**`, Action: lexer.Push("DocComment")},
		},
		"DocComment": {
			{Name: "DocCommentEnd", Pattern: `*/`, Action: lexer.Pop()},
			lexer.Include("DocCommentStart"),
			{Name: "DocCommentContents", Pattern: `([^*/]|/[^*]|\*[^/])+`, Action: nil},
		},
		"BlockCommentStart": {
			{Name: "BlockCommentStart", Pattern: `/*`, Action: lexer.Push("BlockComment")},
		},
		"BlockComment": {
			{Name: "BlockCommentEnd", Pattern: `*/`, Action: lexer.Pop()},
			lexer.Include("BlockCommentStart"),
			{Name: "BlockCommentContents", Pattern: `([^*/]|/[^*]|\*[^/])+`, Action: nil},
		},
		"String": {
			{Name: "StringEnd", Pattern: `"`, Action: lexer.Pop()},
			{Name: "StringInterpolant", Pattern: `\\\(`, Action: lexer.Push("Root")},
			{Name: "StringContents", Pattern: `([^"\\]|\\[^(])+`, Action: nil},
		},
	})
	Parser = participle.MustBuild[Root](
		participle.Lexer(lex),
		participle.Elide("Whitespace", "BlockCommentStart", "BlockCommentContents", "BlockCommentEnd", "LineComment"),
		participle.Unquote("StringContents"),
		participle.UseLookahead(3),
		participle.Union[Statement](Module{}, BindStatement{}, FunctionStatement{}, IfStatement{}),
		participle.Union[Value](Integer{}, String{}),
	)
)

type (
	Root struct {
		Statements []DocumentedStatement `parser:"@@*"`
	}
	DocumentedStatement struct {
		Pos        lexer.Position
		DocComment string    `parser:"(DocCommentStart @(DocCommentContents*) DocCommentEnd)*"`
		Statement  Statement `parser:"@@?"`
	}

	Statement interface{ statement() }
	Block     struct {
		Pos        lexer.Position
		Statements []DocumentedStatement `parser:"'{' @@* '}'"`
	}
	Module struct {
		Pos   lexer.Position
		Name  string `parser:"'module' @Ident"`
		Block Block  `parser:"@@"`
	}
	BindStatement struct {
		Pos       lexer.Position
		IsStatic  bool       `parser:"@'static'?"`
		IsMutable bool       `parser:"'let' @'mut'?"`
		Name      string     `parser:"@Ident"`
		Type      TypeValue  `parser:"(':' @@)?"`
		Expr      Expression `parser:"'=' @@"`
	}
	FunctionStatement struct {
		Pos        lexer.Position
		Name       string              `parser:"'fun' @Ident"`
		Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
		Return     TypeValue           `parser:"(':' @@)?"`
		Block      Block               `parser:"@@"`
	}
	FunctionParameter struct {
		Pos  lexer.Position
		Name string    `parser:"@Ident"`
		Type TypeValue `parser:"':' @@"`
	}
	IfStatement struct {
		Pos        lexer.Position
		Condition  Expression `parser:"'if' @@"`
		ThenBlock  Block      `parser:"@@"`
		ElseBlock  *Block     `parser:"('else' @@)?"`
	}

	Expression    interface{ expression() }
	BindingAccess struct {
		Pos       lexer.Position
		PathParts string `parser:"@Ident ('::' @Ident)* | @Operator"`
	}
	FunctionCall struct {
		Pos       lexer.Position
		Target    Expression   `parser:"@@ '('"`
		Arguments []Expression `parser:"(@@ (',' @@)* ','?)? ')'"`
	}
	Lambda struct {
		Pos        lexer.Position
		Parameters []FunctionParameter `parser:"'fun' '(' (@@ (',' @@)* ','?)? ')'"`
		Return     TypeValue           `parser:"(':' @@)?"`
		Block      Block               `parser:"@@"`
	}
	ValueExpression struct {
		Pos   lexer.Position
		Value Value `parser:"@@"`
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
		Pos         lexer.Position
		Content     string     `parser:"@StringContents"`
		Interpolant Expression `parser:"| StringInterpolant @@ ')'"`
	}

	TypeValue interface {
		Value
		type_value()
	}
	TypeExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'\\' '(' @@ ')'"`
	}
	IntegerType struct {
		Pos     lexer.Position
		Signed  bool   `parser:"(@'i' | 'u')"`
		BitSize uint64 `parser:"@Integer"`
	}
	PointerType struct {
		Pos        lexer.Position
		IsOwned    bool         `parser:"(@'@' | "`
		IsMutable  bool         `parser:"'*' @'mut'? | (@'mut'? "`
		Extent     *ArrayExtent `parser:"@@))"`
		TargetType TypeValue    `parser:"@@"`
	}
	ArrayExtent struct {
		Pos        lexer.Position
		Dimensions []uint64 `parser:"'[' (@Integer (',' @Integer)* ','?)? ']'"`
	}
	FunctionType struct {
		Pos        lexer.Position
		Name       string              `parser:"'fun' @Ident?"`
		Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
		Return     TypeValue           `parser:"(':' @@)? (?! '{')"`
	}
)

func (Module) statement()            {}
func (Block) statement()             {}
func (BindStatement) statement()     {}
func (FunctionStatement) statement() {}
func (IfStatement) statement()      {}

func (BindingAccess) expression()   {}
func (FunctionCall) expression()    {}
func (Lambda) expression()          {}
func (ValueExpression) expression() {}

func (Integer) value()        {}
func (String) value()         {}
func (TypeExpression) value() {}
func (IntegerType) value()    {}
func (PointerType) value()    {}
func (FunctionType) value()   {}

func (TypeExpression) type_value() {}
func (IntegerType) type_value()    {}
func (PointerType) type_value()    {}
func (FunctionType) type_value()   {}
