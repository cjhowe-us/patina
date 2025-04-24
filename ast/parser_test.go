package ast

import (
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
)

func TestLexer(t *testing.T) {
	RunTests(
		t,
		"lexer.json",
		func(input string, name string) ([]lexer.Token, error) {
			lex, err := Lexer.LexString(name, input)
			if err != nil {
				return nil, err
			}
			tokens, err := lexer.ConsumeAll(lex)
			if err != nil {
				return nil, err
			}
			return tokens, nil
		},
		false,
	)
}

func TestParser(t *testing.T) {
	RunTests(
		t,
		"parser.json",
		func(input, name string) (Root, error) {
			root, err := Parser.ParseString(name, input)
			if err != nil || root == nil {
				return Root{}, err
			}
			return *root, nil
		},
		false,
	)
}

func TestParserErrors(t *testing.T) {
	RunTests(
		t,
		"parser_error.json",
		func(input, name string) (Root, error) {
			root, err := Parser.ParseString(name, input)
			if err != nil || root == nil {
				return Root{}, err
			}
			return *root, nil
		},
		true,
	)
}
