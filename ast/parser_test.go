package ast

import (
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
)

type TestToken struct {
	TokenType  string `json:"type"`
	TokenValue string `json:"value"`
}

func TestLexer(t *testing.T) {
	syms := map[lexer.TokenType]string{}
	for str, tok := range Lexer.Symbols() {
		syms[tok] = str
	}

	RunTests(
		t,
		[]string{"lexer.json"},
		func(input string, name string) ([]TestToken, error) {
			lex, err := Lexer.LexString(name, input)
			if err != nil {
				return nil, err
			}
			tokens, err := lexer.ConsumeAll(lex)
			if err != nil {
				return nil, err
			}

			testTokens := make([]TestToken, len(tokens))
			for i, token := range tokens {
				testTokens[i] = TestToken{
					TokenType:  syms[token.Type],
					TokenValue: token.Value,
				}
			}
			return testTokens, nil
		},
	)
}

func TestParser(t *testing.T) {
	RunTests(
		t,
		[]string{
			"parser.json",
			"parser_additional.json",
			"real_world_examples.json",
		},
		func(input, name string) (Root, error) {
			root, err := Parser.ParseString(name, input)
			if err != nil || root == nil {
				return Root{}, err
			}
			return *root, nil
		},
	)
}

func TestParserErrors(t *testing.T) {
	RunTests(
		t,
		[]string{
			"parser_error.json",
			"parser_error_additional.json",
		},
		func(input, name string) (Root, error) {
			root, err := Parser.ParseString(name, input)
			if err != nil || root == nil {
				return Root{}, err
			}
			return *root, nil
		},
	)
}
