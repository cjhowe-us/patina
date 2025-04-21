package ast

import (
	"slices"
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
		participle.Union[Statement](Module{}, FunctionStatement{}, ReturnStatement{}, BreakStatement{}, LetStatement{}, ExpressionStatement{}),
		participle.Union[Expression](IfExpression{}, CaseExpression{}, Lambda{}, BlockExpression{}, TypeExpression{}, Operation{}),
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
