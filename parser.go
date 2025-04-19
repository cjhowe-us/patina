package patina

import (
	"fmt"
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
	keywordType = Lexer.Symbols()["Keyword"]
	keywords    = []string{
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
	stripTokensByType = map[string]struct {
		Prefix string
		Suffix string
	}{
		"Label": {Prefix: ":", Suffix: ""},
		"Tag":   {Prefix: "@", Suffix: ""},
		"Ident": {Prefix: "`", Suffix: "`"},
	}
	strippedTokenTypeToString = map[lexer.TokenType]string{}
	strippedTokenTypeStrings  = []string{}
	Parser                    = participle.MustBuild[Root](
		participle.UseLookahead(4),
		participle.Lexer(Lexer),
		participle.Map(stripTokensAndMapKeywords, strippedTokenTypeStrings...),
		participle.Elide("WS", "EOL", "LineComment", "BlockCommentStart", "BlockCommentEnd", "BlockCommentContents"),
		participle.Union[Statement](Module{}, LetStatement{}, FunctionStatement{}, ReturnStatement{}, BreakStatement{}, ExpressionStatement{}),
		participle.Union[Expression](IfExpression{}, CaseExpression{}, Lambda{}, StaticExpression{}, TypeExpression{}, Block{}, Operation{}),
		participle.Union[Atom](GroupedExpression{}, AtomicType{}, AtomicValue{}, Binding{}),
		participle.Union[Value](Integer{}, String{}),
		participle.Union[Type](IntegerType{}, PointerType{}, ArrayType{}, FunctionType{}),
	)
)

func init() {
	for tokenTypeString := range stripTokensByType {
		tokenType := Lexer.Symbols()[tokenTypeString]
		strippedTokenTypeToString[tokenType] = tokenTypeString
		strippedTokenTypeStrings = append(strippedTokenTypeStrings, tokenTypeString)
	}
}

func stripTokensAndMapKeywords(token lexer.Token) (lexer.Token, error) {
	if slices.Contains(keywords, token.Value) {
		token.Type = keywordType
	}

	typeString, ok := strippedTokenTypeToString[token.Type]
	if !ok {
		return token, nil
	}
	strip, ok := stripTokensByType[typeString]
	if !ok {
		return token, nil
	}
	if strip.Prefix != "" {
		token.Value = strings.TrimPrefix(token.Value, strip.Prefix)
	}
	if strip.Suffix != "" {
		token.Value = strings.TrimSuffix(token.Value, strip.Suffix)
	}
	return token, nil
}

type (
	Root struct {
		Pos        lexer.Position
		Statements []DecoratedStatement `parser:"(@@ (EOL @@)*)? EOF"`
	}
	Block struct {
		Pos        lexer.Position
		Label      string               `parser:"@Label?"`
		Statements []DecoratedStatement `parser:"'{' (@@ (EOL @@)*)? '}'"`
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
		Arguments []Expression `parser:"('(' (@@ (',' @@)* ','?)? ')')?"`
	}
)

type (
	Statement interface {
		statement()
		DefinitionDiscoverer
	}

	Module struct {
		Pos  lexer.Position
		Name string `parser:"'module' @Ident"`
		Body Block  `parser:"@@"`
	}

	FunctionStatement struct {
		Pos        lexer.Position
		IsStatic   bool                `parser:"@'static'?"`
		Name       string              `parser:"'fun' @Ident"`
		Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
		Return     Atom                `parser:"(':' @@)?"`
		Block      Block               `parser:"@@"`
	}
	FunctionParameter struct {
		Pos  lexer.Position
		Name string `parser:"@Ident"`
		Type Atom   `parser:"':' @@"`
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
		Type      Atom       `parser:"(':' @@)?"`
		Value     Expression `parser:"('=' @@)? (?= EOL | EOF)"`
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
		DefinitionDiscoverer
	}

	IfExpression struct {
		Pos     lexer.Position
		If      IfBranch   `parser:"@@"`
		ElseIfs []IfBranch `parser:"('else' @@)*"`
		Else    Expression `parser:"('else' @@)?"`
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
		Parameters []FunctionParameter `parser:"'fun' '(' (@@ (',' @@)* ','?)? ')'"`
		Return     Atom                `parser:"(':' @@)?"`
		Body       Block               `parser:"@@"`
	}

	StaticExpression struct {
		Pos    lexer.Position
		Static Expression `parser:"'static' @@"`
	}
	TypeExpression struct {
		Pos  lexer.Position
		Type Type `parser:"@@"`
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
		Invocation []Expression `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
		Index      Expression   `parser:"| '[' @@ ']'"`
		Operator   string       `parser:"| @Operator"`
	}
)

func (IfExpression) expression()     {}
func (CaseExpression) expression()   {}
func (Lambda) expression()           {}
func (StaticExpression) expression() {}
func (TypeExpression) expression()   {}
func (Block) expression()            {}
func (Operation) expression()        {}

type (
	Atom interface {
		atom()
		DefinitionDiscoverer
	}
	GroupedExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'(' @@ ')'"`
	}
	AtomicType struct {
		Pos  lexer.Position
		Type Type `parser:"@@"`
	}
	AtomicValue struct {
		Pos   lexer.Position
		Value Value `parser:"@@"`
	}
	Binding struct {
		Pos   lexer.Position
		Ident string `parser:"@Ident"`
	}
)

func (GroupedExpression) atom() {}
func (AtomicType) atom()        {}
func (AtomicValue) atom()       {}
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
	Type interface {
		is_type()
	}
	PointerType struct {
		Pos       lexer.Position
		IsOwned   bool `parser:"'*' (@'own'"`
		IsRaw     bool `parser:"| @'raw'"`
		IsMutable bool `parser:"| @'mut')?"`
		Target    Atom `parser:"@@"`
	}
	ArrayType struct {
		Pos       lexer.Position
		Dimension *uint64 `parser:"'[' @Integer? ']'"`
		Element   Atom    `parser:"@@"`
	}
	IntegerType struct {
		Pos     lexer.Position
		Signed  bool
		BitSize uint64
	}
	FunctionType struct {
		Pos        lexer.Position
		Name       string              `parser:"'fun' @Ident?"`
		Parameters []FunctionParameter `parser:"'(' (@@ (',' @@)* ','?)? ')'"`
		Return     Atom                `parser:"(':' @@)? (?! '{')"`
	}
)

func (PointerType) is_type()  {}
func (ArrayType) is_type()    {}
func (IntegerType) is_type()  {}
func (FunctionType) is_type() {}

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
