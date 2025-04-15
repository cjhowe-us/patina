package patina

import (
	"strings"
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
			name: "Label",
			src:  ":label",
		},
		{
			name: "Tag",
			src:  "@ns.tag",
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
		{
			name: "Backtick Ident",
			src:  "`my-ident`",
		},
		{
			name: "Empty String",
			src:  `""`,
		},
		{
			name: "Zero Integer",
			src:  "0",
		},
		{
			name: "Block Comment",
			src:  "/* block comment */",
		},
		{
			name: "Nested Block Comment",
			src:  "/* nested /* block */ comment */",
		},
		{
			name: "String Interpolation",
			src:  `"Hello, \(name)!"`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			spaceless_name := strings.ReplaceAll(tt.name, " ", "_")
			lex, err := Lexer.LexString(spaceless_name, tt.src)
			if err != nil {
				t.Fatalf("Unexpected error: %v\n%s", err, tt.src)
			}
			tokens, err := lexer.ConsumeAll(lex)
			if err != nil {
				t.Fatalf("Unexpected error: %v\n%s", err, tt.src)
			}

			snapsConfig := snaps.WithConfig(jsonSnapshotConfig)
			snapsConfig.MatchJSON(t, tokens)
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
			name: "Block atom with label",
			src: `:my_label {
				let x = 1
			}`,
		},
		{
			name: "Block atom with multiple statements",
			src: `{
				let x = 1
				let y: *i32 = 2
				break x + y
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
			src: `@tag.anno("hello world", 100, \(quoted)) fun add(a: i64, b: i64): i64 {
				let c = a + b
			}`,
		},
		{
			name: "case statement",
			src: `case {
				if (x > 10) 1
			    else 2
			}`,
		},
		{
			name: "Case statement with many branches",
			src: `case {
				if (x == 1) 1
				if (x == 2) 2
				else 3
			}`,
		},
		{
			name: "Using if shortcut with block",
			src: `
			if (x > 10 && y < 20 || z == 30) {
				return x + y + z
			} else if (interlockable()) :test {
			    break :test 5
			} else {
			    takeOver(world)?.destroy()
			}
			`,
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
			name: "Binding",
			src:  "my.`-path`",
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
			src: `
			let x: i64 = 10
			let ptr: *mut i64 = 20
			let arr: [10]i64
			let fn: fun add(a: i64, b: i64): i64 = add
			`,
		},
		{
			name: "Pointer type",
			src:  `let ptr: *mut i64 = 20`,
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
			name: "Indexing",
			src:  `let x = arr[1]`,
		},
		{
			name: "Indexing with prefix op",
			src:  `let x = !arr[1]`,
		},
		{
			name: "Indexing with postfix op",
			src:  `let x = arr[1]?`,
		},
		{
			name: "Indexing with prefix and postfix ops",
			src:  `let x = !arr[1]?`,
		},
		{
			name: "Indexing with prefix, infix, and postfix ops",
			src:  `let x = -arr[1]? + !arr[2]`,
		},
		{
			name: "Indexing with invocations and nesting",
			src:  `let x = arr[1 + 2](1 + 2)?`,
		},
		{
			name: "Simple break",
			src:  `break`,
		},
		{
			name: "Break with label",
			src:  `break :myLabel`,
		},
		{
			name: "Break with expression",
			src:  `break (1 + 2)`,
		},
		{
			name: "Break with label and expression",
			src:  `break :myLabel (3 * 4)`,
		},
		{
			name: "Quoted label and tag",
			src:  "\\(:my_label { @my.tag break :my_label })",
		},
		{
			name: "Quoted binding",
			src:  "\\(my.`-path`)",
		},
		{
			name: "Quoted expression",
			src:  "\\(1 + 2)",
		},
		{
			name: "Doc comment",
			src: `/* My doc comment */
			fun test() {}`,
		},
		{
			name: "Return with tag",
			src:  `return :label`,
		},
		{
			name: "Return with expression",
			src:  `return (1 + 2)`,
		},
		{
			name: "Return with tag and expression",
			src:  `return :label (3 * 4)`,
		},
		{
			name: "Static labeled block expression",
			src: `let x = static :labeled {
				break :labeled 5
			}`,
		},
		{
			name: "Raw pointer type",
			src:  `let x: *raw i32`,
		},
		{
			name: "Owned pointer type",
			src:  `let x: *own i32`,
		},
		{
			name: "Unsized array type",
			src:  `let x: []i32`,
		},
		{
			name: "Named function type",
			src:  `let x: fun MyFunc(a: i32): i32`,
		},
		{
			name: "Empty input",
			src:  "",
		},
		{
			name: "Multiple root statements",
			src:  "let x = 1\nlet y = 2",
		},
		{
			name: "Pointer to array type",
			src:  "let x: *[10]i32",
		},
		{
			name: "Array of function types",
			src:  "let x: []fun(): i32",
		},
		{
			name: "Nested string interpolation",
			src:  `"Hello, \("Mr. \(lastName)")!"`,
		},
		{
			name: "Multiple annotations",
			src:  "@anno1 @anno2(true) fun test() {}",
		},
		{
			name: "Grouped expression",
			src:  "let x = (1 + 2)",
		},
		{
			name: "Lambda with return type",
			src:  "let x = fun(): i32 { return 1 }",
		},
		{
			name: "Function type without parameters",
			src:  "let x: fun(): i32",
		},
		{
			name: "Function type without return type",
			src:  "let x: fun(a: i32)",
		},
		{
			name: "Immutable pointer type",
			src:  "let x: *i32",
		},
		{
			name: "Mutable let with type and expression",
			src:  "let mut x: i32 = 1",
		},
		{
			name: "Static let with type and expression",
			src:  "static let x: i32 = 1",
		},
		{
			name: "Trailing comma in function parameters",
			src:  "fun test(a: i32, ) {}",
		},
		{
			name: "Trailing comma in function call arguments",
			src:  "test(1, )",
		},
		{
			name: "Trailing comma in annotation arguments",
			src:  "@myAnno(1, ) fun test() {}",
		},
		{
			name: "Integer width edge cases",
			src: `let min: i1 = 0
let max: u65535 = 0`,
		},
		{
			name: "Complex nested expressions",
			src:  `let x = (1 + (2 * (3 - (4 / 5))))\nlet y = ((((1 + 2) * 3) - 4) / 5)`,
		},
		{
			name: "Mutable pointer modifiers",
			src:  `let x: *mut i32 = 0`,
		},
		{
			name: "Raw pointer modifiers",
			src:  `let x: *raw i32 = 0`,
		},
		{
			name: "Own pointer modifiers",
			src:  `let x: *own i32 = 0`,
		},
		{
			name: "Multi-line block comment",
			src: `/* This is a
			   multi-line
			   comment with /* nesting */
			   and more text */
			let x = 1`,
		},
		{
			name: "Complex function type",
			src:  `let x: fun(cb: fun(a: i32, b: i32): fun(): i32): i32`,
		},
		{
			name: "Multiple annotations with expressions",
			src:  `@tag1(1 + 2) @tag2(x.y[3]?.z()) fun test() {}`,
		},
		{
			name: "Deeply nested blocks",
			src: `{
				{
					{
						{
							let x = 1
							{
								let y = 2
							}
						}
					}
				}
			}`,
		},
		{
			name: "Empty blocks with labels",
			src: `:outer {
				:inner {}
			}`,
		},
		{
			name: "Complex indexing chain",
			src:  `let x = arr[idx1][idx2][fn()+3]?.member()`,
		},
		{
			name: "All operator types combined",
			src:  `let x = !-~(a + b * c / d % e && f || g) == h != i < j > k <= l >= m?.n()`,
		},
		{
			name: "Static function",
			src: `static fun test(): i32 {
				return 42
			}`,
		},
		{
			name: "Nested array types",
			src:  `let x: [2][3][4]i32`,
		},
		{
			name: "Nested string fragments",
			src:  `let x = "begin \("nest1 \("nest2 \("nest3")")")"`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			spaceless_name := strings.ReplaceAll(tt.name, " ", "_")
			root, err := Parser.ParseString(spaceless_name, tt.src)
			if err != nil {
				t.Fatalf("Unexpected error: %v\n%s", err, tt.src)
			}
			snapsConfig := snaps.WithConfig(jsonSnapshotConfig)
			snapsConfig.MatchJSON(t, root)
		})
	}
}

func TestParserErrors(t *testing.T) {
	type parserErrorTestCase struct {
		name string
		src  string
	}

	tests := []parserErrorTestCase{
		{
			name: "Mismatched parentheses",
			src:  "let x = (1 + 2",
		},
		{
			name: "Mismatched braces",
			src:  "fun test() {",
		},
		{
			name: "Mismatched brackets",
			src:  "let x = arr[1",
		},
		{
			name: "Invalid function parameter syntax",
			src:  "fun test(a b: i32) {}",
		},
		{
			name: "Missing colon in parameter",
			src:  "fun test(a i32) {}",
		},
		{
			name: "Invalid let syntax",
			src:  "let = 5",
		},
		{
			name: "Invalid type syntax",
			src:  "let x: * {}",
		},
		{
			name: "Invalid break syntax",
			src:  "break :",
		},
		{
			name: "Invalid return syntax",
			src:  "return :",
		},
		{
			name: "Invalid module syntax",
			src:  "module {}",
		},
		{
			name: "Incomplete string",
			src:  `let x = "hello`,
		},
		{
			name: "Incomplete string interpolation",
			src:  `let x = "hello \(name"`,
		},
		{
			name: "Multiple modifiers on pointers",
			src:  "let x: *own mut i32",
		},
		{
			name: "Invalid array dimension",
			src:  "let x: [-1]i32",
		},
		{
			name: "Invalid function call",
			src:  "let x = f(,)",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			spaceless_name := strings.ReplaceAll(tt.name, " ", "_")
			_, err := Parser.ParseString(spaceless_name, tt.src)
			if err == nil {
				t.Fatalf("Expected error, but got none\n%s", tt.src)
			}

			snapsConfig := snaps.WithConfig(jsonSnapshotConfig)
			snapsConfig.MatchSnapshot(t, err.Error())
		})
	}
}
