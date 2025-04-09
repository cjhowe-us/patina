package patina

import (
	"regexp"
	"slices"
	"strconv"
	"strings"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
)

var (
	Lexer = lexer.MustStateful(lexer.Rules{
		"Root": {
			lexer.Include("BlockCommentStart"),
			{Name: "WS", Pattern: `[ \t]+`, Action: nil},
			{Name: "EOL", Pattern: `[\n\r]+`, Action: nil},
			{Name: "LineComment", Pattern: `(?://)[^\n]*`, Action: nil},
			{Name: "BracketLeft", Pattern: `[([{]`, Action: lexer.Push("Root")},
			{Name: "BracketRight", Pattern: `[)\]}]`, Action: lexer.Pop()},
			{Name: "Label", Pattern: ":[a-zA-Z0-9_]+", Action: nil},
			{Name: "Tag", Pattern: "@[a-zA-Z0-9_\\.]+", Action: nil},
			{Name: "Ident", Pattern: "[a-zA-Z_][a-zA-Z0-9_]*|`[^`\\s]+`", Action: nil},
			{Name: "Keyword", Pattern: "[a-zA-Z]+", Action: nil},
			{Name: "Symbol", Pattern: `[:,]`, Action: nil},
			{Name: "Operator", Pattern: `[~!@#$%^&*\-=+:<>?/|\\]{2,}|[~!#$%^&*\-=+<>/?\.\\]`, Action: nil},
			{Name: "Integer", Pattern: `\d+`, Action: nil},
			{Name: "StringStart", Pattern: `"`, Action: lexer.Push("String")},
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
	keywords = []string{
		"module",
		"fun",
		"return",
		"break",
		"mut",
		"own",
		"raw",
		"let",
		"if",
		"else",
		"case",
		"static",
	}
	keywordType  = Lexer.Symbols()["Keyword"]
	intTypeRegex = regexp.MustCompile(`^([iu])(\d+)`)
	Parser       = participle.MustBuild[Root](
		participle.UseLookahead(4),
		participle.Lexer(Lexer),
		participle.Map(func(token lexer.Token) (lexer.Token, error) {
			if slices.Contains(keywords, token.Value) {
				token.Type = keywordType
			} else {
				token.Value = strings.TrimPrefix(strings.TrimSuffix(token.Value, "`"), "`")
			}
			return token, nil
		}, "Ident"),
		participle.Elide("WS", "EOL", "LineComment", "BlockCommentStart", "BlockCommentEnd", "BlockCommentContents"),
		participle.Union[Statement](Module{}, FunctionStatement{}, ReturnStatement{}, LetStatement{}, BreakStatement{}, ExpressionStatement{}),
		participle.Union[Expression](QuotedExpression{}, IfExpression{}, CaseExpression{}, Lambda{}, StaticExpression{}, Block{}, PointerType{}, ArrayType{}, IntegerType{}, FunctionType{}, Operation{}),
		participle.Union[Atom](GroupedExpression{}, AtomValue{}, Binding{}),
		participle.Union[TypeValue](PointerType{}, ArrayType{}, IntegerType{}, FunctionType{}),
		participle.Union[Value](Integer{}, String{}, PointerType{}, ArrayType{}, IntegerType{}, FunctionType{}),
	)
)

type (
	Root struct {
		Pos        lexer.Position
		Statements []DecoratedStatement `parser:"[@@ (EOL @@)*] EOF"`
	}
	Block struct {
		Pos        lexer.Position
		Label      string               `parser:"@Label?"`
		Statements []DecoratedStatement `parser:"'{' [@@ (EOL @@)*] '}'"`
	}
	DecoratedStatement struct {
		Pos         lexer.Position
		Comments    *DocComment  `parser:"@@?"`
		Annotations []Annotation `parser:"@@*"`
		Statement   Statement    `parser:"@@"`
	}
	DocComment struct {
		Pos       lexer.Position
		Fragments []DocCommentFragment `parser:"BlockCommentStart @@* BlockCommentEnd"`
	}
	DocCommentFragment struct {
		Pos     lexer.Position
		Text    string      `parser:"@BlockCommentContents"`
		Comment *DocComment `parser:"| @@"`
	}
	Annotation struct {
		Pos       lexer.Position
		Name      string       `parser:"@Tag"`
		Arguments []Expression `parser:"['(' [@@ (',' @@)* ','?] ')']"`
	}
)

type (
	Statement interface{ statement() }

	Module struct {
		Pos  lexer.Position
		Name string `parser:"'module' @Ident"`
		Body Block  `parser:"@@"`
	}

	FunctionStatement struct {
		Pos        lexer.Position
		IsStatic   bool                `parser:"@'static'?"`
		Name       string              `parser:"'fun' @Ident"`
		Parameters []FunctionParameter `parser:"'(' [@@ (',' @@)* ','?] ')'"`
		Return     *TypeValue          `parser:"(':' @@)?"`
		Block      Block               `parser:"@@"`
	}
	ReturnStatement struct {
		Pos   lexer.Position
		Label string     `parser:"'return' @Label?"`
		Value Expression `parser:"@@?"`
	}

	LetStatement struct {
		Pos       lexer.Position
		IsStatic  bool       `parser:"@'static'?"`
		IsMutable bool       `parser:"'let' @'mut'?"`
		Name      string     `parser:"@Ident"`
		Type      TypeValue  `parser:"[':' @@]"`
		Value     Expression `parser:"['=' @@]"`
	}

	BreakStatement struct {
		Pos   lexer.Position
		Label string     `parser:"'break' @Label?"`
		Value Expression `parser:"@@?"`
	}

	ExpressionStatement struct {
		Pos        lexer.Position
		Expression Expression `parser:"@@"`
	}
)

func (Module) statement()              {}
func (FunctionStatement) statement()   {}
func (ReturnStatement) statement()     {}
func (LetStatement) statement()        {}
func (BreakStatement) statement()      {}
func (ExpressionStatement) statement() {}

type (
	Expression interface {
		expression()
	}

	QuotedExpression struct {
		Pos    lexer.Position
		Quoted Expression `parser:"'\\\\' '(' @@ ')'"`
	}

	IfExpression struct {
		Pos     lexer.Position
		If      IfBranch   `parser:"@@"`
		ElseIfs []IfBranch `parser:"('else' @@)*"`
		Else    Expression `parser:"['else' @@]"`
	}
	IfBranch struct {
		Pos       lexer.Position
		Condition Expression `parser:"'if' '(' @@ ')'"`
		Then      Expression `parser:"@@"`
	}

	CaseExpression struct {
		Pos      lexer.Position
		Branches []CaseBranch `parser:"'case' '{' @@+ '}'"`
	}
	CaseBranch struct {
		Pos  lexer.Position
		If   *IfBranch  `parser:"@@"`
		Else Expression `parser:"| 'else' @@"`
	}

	Lambda struct {
		Pos        lexer.Position
		Parameters []FunctionParameter `parser:"'fun' '(' [@@ (',' @@)* ','?] ')'"`
		Return     TypeValue           `parser:"[':' @@]"`
		Body       Block               `parser:"@@"`
	}

	StaticExpression struct {
		Pos    lexer.Position
		Static Expression `parser:"'static' @@"`
	}

	Operation struct {
		Pos      lexer.Position
		Prefix   []string  `parser:"@Operator*"`
		Operands []Operand `parser:"@@+"`
	}
	Operand struct {
		Atom    Atom      `parser:"@@"`
		Postfix []Postfix `parser:"@@*"`
	}
	Postfix struct {
		Pos        lexer.Position
		Invocation []Expression `parser:"'(' [@@ (',' @@)* ','?] ')'"`
		Operator   string       `parser:"| @Operator"`
	}
)

func (QuotedExpression) expression() {}
func (IfExpression) expression()     {}
func (CaseExpression) expression()   {}
func (Lambda) expression()           {}
func (StaticExpression) expression() {}
func (Block) expression()            {}
func (Operation) expression()        {}

type (
	Atom              interface{ atom() }
	GroupedExpression struct {
		Pos        lexer.Position
		Expression Operation `parser:"'(' @@ ')'"`
	}
	AtomValue struct {
		Pos   lexer.Position
		Value Value `parser:"@@"`
	}
	Binding struct {
		Pos   lexer.Position
		Ident string `parser:"@Ident"`
	}
)

func (GroupedExpression) atom() {}
func (AtomValue) atom()         {}
func (Binding) atom()           {}

type (
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
		Text          string     `parser:"@StringFragment+"`
		Interpolation Expression `parser:"| StringInterpolant @@ ')'"`
	}
)

func (Integer) value() {}
func (String) value()  {}

type (
	TypeValue interface {
		Expression
		Value
		type_value()
	}
	PointerType struct {
		Pos        lexer.Position
		IsOwned    bool      `parser:"'*' (@'own'"`
		IsRaw      bool      `parser:"| @'raw'"`
		IsMutable  bool      `parser:"| @'mut')?"`
		TargetType TypeValue `parser:"@@"`
	}
	ArrayType struct {
		Pos         lexer.Position
		Dimension   *uint64   `parser:"'[' @Integer? ']'"`
		ElementType TypeValue `parser:"@@"`
	}
	IntegerType struct {
		Pos     lexer.Position
		Signed  bool
		BitSize uint64
	}
	FunctionType struct {
		Pos        lexer.Position
		Name       string              `parser:"'fun' @Ident?"`
		Parameters []FunctionParameter `parser:"'(' [@@ (',' @@)* ','?] ')'"`
		Return     *TypeValue          `parser:"[':' @@] (?! '{')"`
	}
	FunctionParameter struct {
		Pos  lexer.Position
		Name string    `parser:"@Ident"`
		Type TypeValue `parser:"':' @@"`
	}
)

func (PointerType) expression()  {}
func (ArrayType) expression()    {}
func (IntegerType) expression()  {}
func (FunctionType) expression() {}
func (PointerType) type_value()  {}
func (ArrayType) type_value()    {}
func (IntegerType) type_value()  {}
func (FunctionType) type_value() {}
func (PointerType) value()       {}
func (ArrayType) value()         {}
func (IntegerType) value()       {}
func (FunctionType) value()      {}

func (it *IntegerType) Parse(lex *lexer.PeekingLexer) error {
	token := lex.Peek()
	matches := intTypeRegex.FindAllStringSubmatch(token.Value, -1)
	if matches == nil || len(matches) != 1 || len(matches[0]) != 3 {
		return participle.NextMatch
	}
	sign, size := matches[0][1], matches[0][2]
	bitSize, err := strconv.ParseUint(size, 10, 16)
	if err != nil {
		return err
	}
	lex.Next()
	it.Pos = token.Pos
	it.Signed = strings.Compare(sign, "i") == 0
	it.BitSize = bitSize
	return nil
}
