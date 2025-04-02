package patina

import (
	"regexp"
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
			{Name: "EOL", Pattern: `\n|\r|\r\n`, Action: nil},
			{Name: "LineComment", Pattern: `(?://)[^\n]*`, Action: nil},
			{Name: "BracketLeft", Pattern: `[([{]`, Action: lexer.Push("Root")},
			{Name: "BracketRight", Pattern: `[)\]}]`, Action: lexer.Pop()},
			{Name: "Tag", Pattern: "(?:#)([a-zA-Z0-9_]+|(?:`)[^`\\s]+(?:`))", Action: nil},
			{Name: "Ident", Pattern: "[a-zA-Z_][a-zA-Z0-9_]*|(?:`)[^`\\s]+(?:`)", Action: nil},
			{Name: "Symbol", Pattern: `[:,]`, Action: nil},
			{Name: "PrefixOperator", Pattern: `[~!@$\-\\]`, Action: nil},
			{Name: "PostfixOperator", Pattern: `[?]`, Action: nil},
			{Name: "Operator", Pattern: `[~!@$%^&*\-=+;:<>?/|\\]{2,}|[%^&*\-=+<>/\.]`, Action: nil},
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
	intTypeRegex = regexp.MustCompile(`^([iu])(\d+)`)
	Parser       = participle.MustBuild[Root](
		participle.Lexer(Lexer),
		participle.UseLookahead(10),
		participle.Elide("WS", "EOL", "LineComment", "BlockCommentStart", "BlockCommentEnd", "BlockCommentContents"),
		participle.Union[Statement](Module{}, LetStatement{}, FunctionStatement{}, IfStatement{}, BreakStatement{}, FunctionStatement{}, ReturnStatement{}, ExpressionStatement{}),
		participle.Union[Atom](QuotedExpression{}, GroupedExpression{}, BlockAtom{}, TagAtom{}, Lambda{}, AtomValue{}, Binding{}),
		participle.Union[TypeValue](IntegerType{}, PointerType{}, FunctionType{}, ArrayType{}, TypeExpression{}),
		participle.Union[Value](Integer{}, String{}, IntegerType{}, PointerType{}, FunctionType{}, ArrayType{}, TypeExpression{}),
	)
)

type (
	Root struct {
		Pos        lexer.Position
		Statements []DecoratedStatement `parser:"@@* EOF?"`
	}

	Statement          interface{ statement() }
	DocCommentFragment struct {
		Pos     lexer.Position
		Text    string      `parser:"@BlockCommentContents"`
		Comment *DocComment `parser:"| @@"`
	}
	DocComment struct {
		Pos       lexer.Position
		Fragments []DocCommentFragment `parser:"BlockCommentStart @@* BlockCommentEnd"`
	}
	Annotation struct {
		Pos       lexer.Position
		Name      TagAtom      `parser:"@@"`
		Arguments []Expression `parser:"'(' [@@ (',' @@)* ','?] ')'"`
	}
	DecoratedStatement struct {
		Pos         lexer.Position
		Comments    *DocComment  `parser:"@@?"`
		Annotations []Annotation `parser:"@@*"`
		Statement   Statement    `parser:"@@ (?= EOL | EOF | BracketLeft | BracketRight)"`
	}
	Block struct {
		Pos        lexer.Position
		Statements []DecoratedStatement `parser:"'{' @@* '}'"`
	}
	BreakStatement struct {
		Pos  lexer.Position
		Tag  *TagAtom `parser:"'break' @@?"`
		Atom Atom     `parser:"@@?"`
	}
	Module struct {
		Pos   lexer.Position
		Ident string `parser:"'module' @Ident"`
		Body  Block  `parser:"@@"`
	}
	LetStatement struct {
		Pos       lexer.Position
		IsStatic  bool        `parser:"@'static'?"`
		IsMutable bool        `parser:"'let' @'mut'?"`
		Ident     string      `parser:"@Ident"`
		Type      *TypeValue  `parser:"([':' @@]"`
		Expr      *Expression `parser:"['=' @@])!"`
	}
	FunctionStatement struct {
		Pos        lexer.Position
		Ident      string              `parser:"'fun' @Ident"`
		Parameters []FunctionParameter `parser:"'(' [@@ (',' @@)* ','?] ')'"`
		Return     *TypeValue          `parser:"(':' @@)?"`
		Block      Block               `parser:"@@"`
	}
	ReturnStatement struct {
		Pos        lexer.Position
		Tag        *TagAtom `parser:"'return' @@?"`
		Expression Atom     `parser:"@@?"`
	}
	IfStatement struct {
		Pos              lexer.Position
		Condition        Expression    `parser:"'if' @@ "`
		ThenBlock        Block         `parser:"@@"`
		ElseIfStatements []IfStatement `parser:"('else' @@)*"`
		ElseBlock        *Block        `parser:"['else' @@]"`
	}
	ExpressionStatement struct {
		Pos        lexer.Position
		Expression Expression `parser:"@@"`
	}

	Expression struct {
		Pos       lexer.Position
		Term      Term              `parser:"@@"`
		BinaryOps []BinaryOperation `parser:"@@*"`
	}
	BinaryOperation struct {
		Pos      lexer.Position
		Operator string `parser:"@(PrefixOperator | Operator)"`
		Term     Term   `parser:"@@"`
	}
	Term struct {
		Pos         lexer.Position
		PrefixOps   []string     `parser:"@PrefixOperator*"`
		Operand     Atom         `parser:"@@"`
		Invocations []Invocation `parser:"@@*"`
	}
	Invocation struct {
		Pos        lexer.Position
		Arguments  []Expression `parser:"(['(' [@@ (',' @@)* ','?] ')']"`
		PostfixOps []string     `parser:"@PostfixOperator*)!"`
	}

	Atom             interface{ atom() }
	QuotedExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'\\\\' '(' @@ ')'"`
	}
	GroupedExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'(' @@ ')'"`
	}
	Lambda struct {
		Pos        lexer.Position
		Parameters []FunctionParameter `parser:"'fun' '(' [@@ (',' @@)* ','?] ')'"`
		Return     *TypeValue          `parser:"[':' @@]"`
		Block      Block               `parser:"@@"`
	}
	BlockAtom struct {
		Pos      lexer.Position
		Tag      *TagAtom `parser:"[@@ ':']"`
		IsStatic bool     `parser:"@'static'?"`
		Body     Block    `parser:"@@"`
	}
	TagAtom struct {
		Pos lexer.Position
		Tag string `parser:"@Tag"`
	}
	AtomValue struct {
		Pos   lexer.Position
		Value Value `parser:"@@"`
	}
	Binding struct {
		Pos   lexer.Position
		Ident string `parser:"@Ident"`
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
		Signed  bool
		BitSize uint64
	}
	PointerType struct {
		Pos        lexer.Position
		IsOwned    bool      `parser:"(@'@'"`
		IsMutable  bool      `parser:"| '*' @'mut'?)"`
		TargetType TypeValue `parser:"@@"`
	}
	ArrayType struct {
		Pos         lexer.Position
		Dimension   *uint64   `parser:"'[' @Integer? ']'"`
		ElementType TypeValue `parser:"@@"`
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
	TypeExpression struct {
		Pos        lexer.Position
		Expression Expression `parser:"'\\\\' '(' @@ ')'"`
	}
)

func (Module) statement()              {}
func (Block) statement()               {}
func (BreakStatement) statement()      {}
func (LetStatement) statement()        {}
func (FunctionStatement) statement()   {}
func (ReturnStatement) statement()     {}
func (IfStatement) statement()         {}
func (ExpressionStatement) statement() {}

func (QuotedExpression) atom()  {}
func (GroupedExpression) atom() {}
func (BlockAtom) atom()         {}
func (TagAtom) atom()           {}
func (Lambda) atom()            {}
func (AtomValue) atom()         {}
func (Binding) atom()           {}

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
