package patina

import (
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/gkampitakis/go-snaps/snaps"
)

func TestLexer(t *testing.T) {
	type lexerTestCase struct {
		name string
		src  string
	}

	tests := []lexerTestCase{
		{
			name: "Tag",
			src:  "#tag",
		},
		{
			name: "Integer",
			src:  "123",
		},
		{
			name: "String",
			src:  `"Hello, world!"`,
		},
		{
			name: "Prefix operator",
			src:  "~",
		},
		{
			name: "Postfix operator",
			src:  "?",
		},
		{
			name: "Infix operator",
			src:  "+",
		},
		{
			name: "Parentheses",
			src:  "()",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			lex, err := Lexer.LexString(tt.name, tt.src)
			if err != nil {
				t.Fatalf("Unexpected error: %v\n%s", err, tt.src)
			}
			tokens, err := lexer.ConsumeAll(lex)
			if err != nil {
				t.Fatalf("Unexpected error: %v\n%s", err, tt.src)
			}
			snaps.MatchYAML(t, tokens)
		})
	}
}

func TestParser(t *testing.T) {
	type parserTestCase struct {
		name string
		src  string
	}

	tests := []parserTestCase{
		{
			name: "Module",
			src: `module test {
				let x = *mut i32
			}`,
		},
		{
			name: "Simple binding",
			src:  "let x = 1",
		},
		{
			name: "Mutable binding",
			src:  "let mut y = 2",
		},
		{
			name: "Static binding",
			src:  "static let z = 3",
		},
		{
			name: "Binding with type",
			src:  "let a: i64 = 4",
		},
		{
			name: "Binding without init",
			src:  "let b: i64",
		},
		{
			name: "Block atom",
			src: `{
				let x = 1
			}`,
		},
		{
			name: "Block atom with tag",
			src: `#myTag: {
				let x = 1
			}`,
		},
		{
			name: "Function statement",
			src: `fun add(a: i64, b: u64): i64 {
				let c = a + b
				return c
			}`,
		},
		{
			name: "Function statement with annotation",
			src: `#myTag("hello world", 100, \#quoted) fun add(a: i64, b: i64): i64 {
				let c = a + b
			}`,
		},
		{
			name: "If statement",
			src: `if x > 10 {
				let y = 1
			} else {
				let y = 2
			}`,
		},
		{
			name: "String literal",
			src:  `let msg = "Hello, world!"`,
		},
		{
			name: "String interpolation",
			src:  `let msg = "Hello, \(name)!"`,
		},
		{
			name: "Comments",
			src: `// Line comment
			/* Block comment */
			fun test() {
			}`,
		},
		{
			name: "Integer type",
			src:  `let x = i64`,
		},
		{
			name: "Type value",
			src: `let x: i64 = 10
			let ptr: *mut i64 = 20
			let arr: [10]i64
			let fn: fun add(a: i64, b: i64): i64 = add`,
		},
		{
			name: "Lambda expression",
			src: `let adder = fun (b: i32) {
				let a = 1 + b
			}`,
		},
		{
			name: "Binary expression",
			src:  `let x = 1 + 2 * 3`,
		},
		{
			name: "Complex if statement",
			src: `if x > 10 && y < 20 || z == 30 {
				let result = x + y + z
			}`,
		},
		{
			name: "Nested expressions",
			src:  `let x = (1 + 2) * 3`,
		},
		{
			name: "Prefix ops",
			src:  `let x = !true`,
		},
		{
			name: "Postfix ops",
			src:  `let x = true?`,
		},
		{
			name: "Prefix and postfix ops",
			src:  `let x = !true?`,
		},
		{
			name: "Invocation",
			src:  `let x = add(1, 2)`,
		},
		{
			name: "Invocation with postfix op",
			src:  `let x = add(1, 2)?`,
		},
		{
			name: "Invocation with prefix op",
			src:  `let x = !add(1, 2)`,
		},
		{
			name: "Invocation with prefix and postfix ops",
			src:  `let x = !add(1, 2)?`,
		},
		{
			name: "Invocation with prefix, infix, and postfix ops",
			src:  `let x = -add(1, 2)? + !sub(3, 4)`,
		},
		{
			name: "If-else statement",
			src: `if x > 10 {
				let z = 1
			} else {
				let z = 3
			}`,
		},
		{
			name: "Simple break",
			src:  `break`,
		},
		{
			name: "Break with label",
			src:  `break #myLabel`,
		},
		{
			name: "Break with expression",
			src:  `break (1 + 2)`,
		},
		{
			name: "Break with label and expression",
			src:  `break #myLabel (3 * 4)`,
		},
		{
			name: "Quoted tag",
			src:  "\\#`myTag`",
		},
		{
			name: "Quoted path",
			src:  "\\my.`-path`",
		},
		{
			name: "Quoted expression",
			src:  "\\(1 + 2)",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			root, err := Parser.ParseString(tt.name, tt.src)
			if err != nil {
				t.Fatalf("Unexpected error: %v\n%s", err, tt.src)
			}
			snaps.MatchYAML(t, root)
		})
	}
}
