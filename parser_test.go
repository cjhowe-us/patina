package patina

import (
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

func TestParser_Module(t *testing.T) {
	src := `module test {
		let x = *mut i32
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_BindStatement(t *testing.T) {
	tests := []struct {
		name     string
		src      string
		isStatic bool
		isMut    bool
		varName  string
		hasType  bool
		hasExpr  bool
	}{
		{
			name:     "Simple binding",
			src:      "let x = 1",
			isStatic: false,
			isMut:    false,
			varName:  "x",
			hasType:  false,
			hasExpr:  true,
		},
		{
			name:     "Mutable binding",
			src:      "let mut y = 2",
			isStatic: false,
			isMut:    true,
			varName:  "y",
			hasType:  false,
			hasExpr:  true,
		},
		{
			name:     "Static binding",
			src:      "static let z = 3",
			isStatic: true,
			isMut:    false,
			varName:  "z",
			hasType:  false,
			hasExpr:  true,
		},
		{
			name:     "With type",
			src:      "let a: i64 = 4",
			isStatic: false,
			isMut:    false,
			varName:  "a",
			hasType:  true,
			hasExpr:  true,
		},
		{
			name:     "Without init",
			src:      "let b: i64",
			isStatic: false,
			isMut:    false,
			varName:  "b",
			hasType:  true,
			hasExpr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			root, err := Parser.ParseString("", tt.src)
			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}
			snaps.MatchJSON(t, root)
		})
	}
}

func TestParser_FunctionStatement(t *testing.T) {
	src := `fun add(a: i64, b: i64): i64 {
		let c = a + b
	}`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_IfStatement(t *testing.T) {
	src := `if x > 10 {
		let y = 1
	} else {
		let y = 2
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_String(t *testing.T) {
	src := `let msg = "Hello, world!"`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_StringInterpolation(t *testing.T) {
	src := `let msg = "Hello, \(name)!"`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_Comments(t *testing.T) {
	src := `// Line comment
	/* Block comment */
	fun test() {
		// Another comment
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_IntegerType(t *testing.T) {
	src := `let x = i64`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_TypeValue(t *testing.T) {
	src := `let x: i64 = 10
	let ptr: *mut i64 = 20
	let arr: [10]i64
	let fn: fun add(a: i64, b: i64): i64 = add`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_FunctionCall(t *testing.T) {
	src := `let result = add(1, 2 + 3)`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_Lambda(t *testing.T) {
	src := `let adder = fun (b: i32) {
		let a = 1 + b
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_BinaryExpression(t *testing.T) {
	src := `let x = 1 + 2 * 3`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_ComplexIfStatement(t *testing.T) {
	src := `if x > 10 && y < 20 || z == 30 {
		let result = x + y + z
	}`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_NestedExpressions(t *testing.T) {
	src := `let x = (1 + 2) * 3`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}

func TestParser_IfElseStatement(t *testing.T) {
	src := `if x > 10 {
		let z = 1
	} else {
		let z = 3
	}`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	snaps.MatchJSON(t, root)
}
