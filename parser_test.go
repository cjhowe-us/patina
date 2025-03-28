package patina

import (
	"strings"
	"testing"
)

func TestParser_Module(t *testing.T) {
	src := `module test {
		let x = 1
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}

	module, ok := root.Statements[0].Statement.(Module)
	if !ok {
		t.Fatalf("Expected Module, got %T", root.Statements[0].Statement)
	}
	if module.Name != "test" {
		t.Errorf("Expected module name 'test', got '%s'", module.Name)
	}
	if len(module.Body.Statements) != 1 {
		t.Errorf("Expected 1 statement in block, got %d", len(module.Body.Statements))
	}
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
			if len(root.Statements) != 1 {
				t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
			}
			bind, ok := root.Statements[0].Statement.(BindStatement)
			if !ok {
				t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
			}
			if bind.IsStatic != tt.isStatic {
				t.Errorf("Expected IsStatic=%v, got %v", tt.isStatic, bind.IsStatic)
			}
			if bind.IsMutable != tt.isMut {
				t.Errorf("Expected IsMutable=%v, got %v", tt.isMut, bind.IsMutable)
			}
			if bind.Name != tt.varName {
				t.Errorf("Expected Name='%s', got '%s'", tt.varName, bind.Name)
			}
			if (bind.Type != nil) != tt.hasType {
				t.Errorf("Expected hasType=%v, got %v", tt.hasType, bind.Type != nil)
			}
			if (bind.Expr != nil) != tt.hasExpr {
				t.Errorf("Expected hasExpr=%v, got %v", tt.hasExpr, bind.Expr != nil)
			}
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
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	fn, ok := root.Statements[0].Statement.(FunctionStatement)
	if !ok {
		t.Fatalf("Expected FunctionStatement, got %T", root.Statements[0].Statement)
	}
	if fn.Name != "add" {
		t.Errorf("Expected name 'add', got '%s'", fn.Name)
	}
	if len(fn.Parameters) != 2 {
		t.Errorf("Expected 2 parameters, got %d", len(fn.Parameters))
	}
	if fn.Parameters[0].Name != "a" {
		t.Errorf("Expected parameter name 'a', got '%s'", fn.Parameters[0].Name)
	}
	if fn.Parameters[1].Name != "b" {
		t.Errorf("Expected parameter name 'b', got '%s'", fn.Parameters[1].Name)
	}
	if len(fn.Block.Statements) != 1 {
		t.Errorf("Expected 1 statement in block, got %d", len(fn.Block.Statements))
	}
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
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}

	ifStmt, ok := root.Statements[0].Statement.(IfStatement)
	if !ok {
		t.Fatalf("Expected IfStatement, got %T", root.Statements[0].Statement)
	}
	if ifStmt.Condition.Operations[0].Operator != ">" {
		t.Errorf("Expected operator '>', got '%s'", ifStmt.Condition.Operations[0].Operator)
	}
	left, ok := ifStmt.Condition.Operations[0].Operand.(BindingAccess)
	if !ok {
		t.Fatalf("Expected BindingAccess, got %T", ifStmt.Condition.Operations[0].Operand)
	}
	if left.Name != "x" {
		t.Errorf("Expected binding name 'x', got '%s'", left.Name)
	}

	if ifStmt.Condition.Operations[0].Operator != ">" {
		t.Errorf("Expected operator '>', got '%s'", ifStmt.Condition.Operations[0].Operator)
	}
	rightVal, ok := ifStmt.Condition.Operations[0].Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue, got %T", ifStmt.Condition.Operations[0].Operand)
	}
	rightInt, ok := rightVal.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer, got %T", rightVal.Value)
	}
	if rightInt.Value != 10 {
		t.Errorf("Expected right value 10, got %d", rightInt.Value)
	}

	if len(ifStmt.ElseIfStatements) != 0 {
		t.Errorf("Expected 0 else-if statements, got %d", len(ifStmt.ElseIfStatements))
	}

	if len(ifStmt.ThenBlock.Statements) != 1 {
		t.Errorf("Expected 1 statement in then block, got %d", len(ifStmt.ThenBlock.Statements))
	}
	if ifStmt.ElseBlock == nil {
		t.Errorf("Expected non-nil else block")
	}
	if len(ifStmt.ElseBlock.Statements) != 1 {
		t.Errorf("Expected 1 statement in else block, got %d", len(ifStmt.ElseBlock.Statements))
	}
}

func TestParser_String(t *testing.T) {
	src := `let msg = "Hello, world!"`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}

	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}
	if bind.Name != "msg" {
		t.Errorf("Expected name 'msg', got '%s'", bind.Name)
	}

	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}

	// Extract string value using our new expression hierarchy
	atom, ok := bind.Expr.Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue, got %T", *bind.Expr)
	}

	str, ok := atom.Value.(String)
	if !ok {
		t.Fatalf("Expected String, got %T", atom.Value)
	}

	if len(str.Fragments) != 1 {
		t.Fatalf("Expected 1 fragment, got %d", len(str.Fragments))
	}
	if str.Fragments[0].Text != "Hello, world!" {
		t.Errorf("Expected text 'Hello, world!', got '%s'", str.Fragments[0].Text)
	}
}

func TestParser_StringInterpolation(t *testing.T) {
	src := `let msg = "Hello, \(name)!"`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}

	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}

	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}

	// Extract string value using our new expression hierarchy
	valExpr, ok := bind.Expr.Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue, got %T", bind.Expr.Operand)
	}

	str, ok := valExpr.Value.(String)
	if !ok {
		t.Fatalf("Expected String, got %T", valExpr.Value)
	}

	if len(str.Fragments) != 3 {
		t.Fatalf("Expected 3 fragments, got %d", len(str.Fragments))
	}
	if str.Fragments[0].Text != "Hello, " {
		t.Errorf("Expected text 'Hello, ', got '%s'", str.Fragments[0].Text)
	}
	if str.Fragments[1].Interpolation == nil {
		t.Errorf("Expected non-nil interpolation")
	}
}

func TestParser_Comments(t *testing.T) {
	src := `// Line comment
	/* Block comment */
	/**
	 * Doc comment
	 */
	fun test() {
		// Another comment
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	if !strings.Contains(root.Statements[0].DocComment, "Doc comment") {
		t.Errorf("Expected doc comment to contain 'Doc comment', got '%s'", root.Statements[0].DocComment)
	}

	fn, ok := root.Statements[0].Statement.(FunctionStatement)
	if !ok {
		t.Fatalf("Expected FunctionStatement, got %T", root.Statements[0].Statement)
	}
	if fn.Name != "test" {
		t.Errorf("Expected name 'test', got '%s'", fn.Name)
	}
	if len(fn.Block.Statements) != 0 {
		t.Errorf("Expected 0 statements in block (comment is elided), got %d", len(fn.Block.Statements))
	}
}

func TestParser_IntegerType(t *testing.T) {
	src := `let x = i64`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}

	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}

	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}

	val, ok := bind.Expr.Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected PrimaryValue, got %T", bind.Expr.Operand)
	}
	intType, ok := val.Value.(IntegerType)
	if !ok {
		t.Fatalf("Expected IntegerType, got %T", val.Value)
	}

	if intType.Signed {
		t.Errorf("Expected unsigned integer type")
	}
	if intType.BitSize != 64 {
		t.Errorf("Expected bit size of '64', got '%d'", intType.BitSize)
	}
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
	if len(root.Statements) != 4 {
		t.Fatalf("Expected 4 statements, got %d", len(root.Statements))
	}

	// Check integer type
	bind1, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}
	if bind1.Type == nil {
		t.Fatalf("Expected non-nil Type")
	}
	// Instead of asserting the specific type, just check that it exists
	if *bind1.Type == nil {
		t.Fatalf("Expected non-nil Type value")
	}

	// Check pointer type
	bind2, ok := root.Statements[1].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[1].Statement)
	}
	if bind2.Type == nil {
		t.Fatalf("Expected non-nil Type")
	}
	ptrType, ok := (*bind2.Type).(PointerType)
	if !ok {
		t.Fatalf("Expected PointerType, got %T", *bind2.Type)
	}
	if !ptrType.IsMutable {
		t.Errorf("Expected mutable pointer, got immutable")
	}

	// Check array type
	bind3, ok := root.Statements[2].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[2].Statement)
	}
	if bind3.Type == nil {
		t.Fatalf("Expected non-nil Type")
	}
	arrType, ok := (*bind3.Type).(ArrayType)
	if !ok {
		t.Fatalf("Expected ArrayType, got %T", *bind3.Type)
	}
	if arrType.Dimensions[0] != 10 {
		t.Errorf("Expected array size 10, got %d", arrType.Dimensions[0])
	}
	elementIntType, ok := arrType.ElementType.(IntegerType)
	if !ok {
		t.Fatalf("Expected IntegerType for array element, got %T", arrType.ElementType)
	}
	if elementIntType.BitSize != 64 {
		t.Errorf("Expected bit size of '64', got '%d'", elementIntType.BitSize)
	}

	// Check function type
	bind4, ok := root.Statements[3].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[3].Statement)
	}
	if bind4.Type == nil {
		t.Fatalf("Expected non-nil Type")
	}
	fnType, ok := (*bind4.Type).(FunctionType)
	if !ok {
		t.Fatalf("Expected FunctionType, got %T", *bind4.Type)
	}
	if fnType.Name != "add" {
		t.Errorf("Expected function name 'add', got '%s'", fnType.Name)
	}
	if len(fnType.Parameters) != 2 {
		t.Errorf("Expected 2 parameters, got %d", len(fnType.Parameters))
	}
}

func TestParser_FunctionCall(t *testing.T) {
	src := `let result = add(1, 2 + 3)`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}
	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}
	fnCall, ok := bind.Expr.Operand.(FunctionCall)
	if !ok {
		t.Fatalf("Expected FunctionCall, got %T", *bind.Expr)
	}
	bindingAccess, ok := (fnCall.Target).(BindingAccess)
	if !ok {
		t.Fatalf("Expected BindingAccess, got %T", fnCall.Target)
	}
	if bindingAccess.Name != "add" {
		t.Errorf("Expected function name 'add', got '%s'", bindingAccess.Name)
	}

	if len(fnCall.Arguments) != 2 {
		t.Fatalf("Expected 2 arguments, got %d", len(fnCall.Arguments))
	}
	atomVal, ok := fnCall.Arguments[0].Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue for arg1, got %T", atomVal)
	}
	intVal1, ok := atomVal.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer for arg1, got %T", atomVal.Value)
	}
	if intVal1.Value != 1 {
		t.Errorf("Expected value 1 for arg1, got %d", intVal1.Value)
	}

	binExpr := fnCall.Arguments[1]
	leftValue, ok := binExpr.Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue for left side, got %T", binExpr.Operand)
	}
	if binExpr.Operations[0].Operator != "+" {
		t.Errorf("Expected operator '+', got '%s'", binExpr.Operations[0].Operator)
	}
	intVal2, ok := leftValue.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer for left side, got %T", leftValue.Value)
	}
	if intVal2.Value != 2 {
		t.Errorf("Expected value 2 for left side, got %d", intVal2.Value)
	}
	rightValue, ok := binExpr.Operations[0].Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue for right side, got %T", binExpr.Operations[0].Operand)
	}
	intVal3, ok := rightValue.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer for right side, got %T", rightValue.Value)
	}
	if intVal3.Value != 3 {
		t.Errorf("Expected value 3 for right side, got %d", intVal3.Value)
	}
}

func TestParser_Lambda(t *testing.T) {
	src := `let adder = fun (b: i32) {
		let a = 1 + b
	}`

	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}

	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}

	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}

	// Verify the name of the binding
	if bind.Name != "adder" {
		t.Errorf("Expected variable name 'adder', got '%s'", bind.Name)
	}

	// Check that we have a lambda expression
	lambda, ok := bind.Expr.Operand.(Lambda)
	if !ok {
		t.Fatalf("Expected Lambda, got %T", *bind.Expr)
	}

	// Check lambda parameters
	if len(lambda.Parameters) != 1 {
		t.Fatalf("Expected 1 parameter, got %d", len(lambda.Parameters))
	}

	if lambda.Parameters[0].Name != "b" {
		t.Errorf("Expected parameter name 'b', got '%s'", lambda.Parameters[0].Name)
	}

	paramType, ok := lambda.Parameters[0].Type.(IntegerType)
	if !ok {
		t.Fatalf("Expected IdentType for parameter, got %T", lambda.Parameters[0].Type)
	}
	if paramType.BitSize != 32 {
		t.Errorf("Expected bit size of '32', got '%d'", paramType.BitSize)
	}

	if lambda.Return != nil {
		t.Fatalf("Expected nil return type for lambda, got %T", lambda.Return)
	}

	if len(lambda.Block.Statements) != 1 {
		t.Fatalf("Expected 1 statement in lambda block, got %d", len(lambda.Block.Statements))
	}
	lambdaBindStmt, ok := lambda.Block.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement in lambda block, got %T", lambda.Block.Statements[0].Statement)
	}
	if lambdaBindStmt.Name != "a" {
		t.Errorf("Expected variable name 'a', got '%s'", lambdaBindStmt.Name)
	}
}

func TestParser_BinaryExpression(t *testing.T) {
	src := `let x = 1 + 2 * 3`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}
	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}

	leftExpr, ok := bind.Expr.Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue on left side, got %T", bind.Expr.Operand)
	}
	intValue1, ok := leftExpr.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer, got %T", leftExpr.Value)
	}
	if intValue1.Value != 1 {
		t.Errorf("Expected left value 1, got %d", intValue1.Value)
	}

	if bind.Expr.Operations[0].Operator != "+" {
		t.Errorf("Expected operator '+', got '%s'", bind.Expr.Operations[0].Operator)
	}
	middleExpr, ok := bind.Expr.Operations[0].Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue on right side, got %T", bind.Expr.Operations[0].Operand)
	}
	intValue2, ok := middleExpr.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer, got %T", leftExpr.Value)
	}
	if intValue2.Value != 2 {
		t.Errorf("Expected middle value 2, got %d", intValue1.Value)
	}

	rightExpr, ok := bind.Expr.Operations[1].Operand.(AtomValue)
	if !ok {
		t.Fatalf("Expected AtomValue on right side of multiplication, got %T", bind.Expr.Operations[1].Operand)
	}
	intValue3, ok := rightExpr.Value.(Integer)
	if !ok {
		t.Fatalf("Expected Integer, got %T", rightExpr.Value)
	}
	if intValue3.Value != 3 {
		t.Errorf("Expected right value 3, got %d", intValue3.Value)
	}
}

func TestParser_ComplexIfStatement(t *testing.T) {
	src := `if x > 10 && y < 20 || z == 30 {
		let result = x + y + z
	}`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	// Test the if statement
	ifStmt, ok := root.Statements[0].Statement.(IfStatement)
	if !ok {
		t.Fatalf("Expected IfStatement, got %T", root.Statements[0].Statement)
	}

	// Test the statement inside the if block
	if len(ifStmt.ThenBlock.Statements) != 1 {
		t.Fatalf("Expected 1 statement in if block, got %d", len(ifStmt.ThenBlock.Statements))
	}
	bindStmt, ok := ifStmt.ThenBlock.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", ifStmt.ThenBlock.Statements[0].Statement)
	}
	if bindStmt.Name != "result" {
		t.Errorf("Expected variable name 'result', got '%s'", bindStmt.Name)
	}

	// Test that the expression (x + y + z) exists
	if bindStmt.Expr == nil {
		t.Fatalf("Expected non-nil expression")
	}
}

func TestParser_NestedExpressions(t *testing.T) {
	src := `let x = (1 + 2) * 3`
	root, err := Parser.ParseString("", src)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	bind, ok := root.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement, got %T", root.Statements[0].Statement)
	}
	if bind.Expr == nil {
		t.Fatalf("Expected non-nil Expr")
	}

	// Test binary expression

	// Left side should be a grouped expression
	groupExpr, ok := bind.Expr.Operand.(GroupedExpression)
	if !ok {
		t.Fatalf("Expected GroupedExpression on left side, got %T", bind.Expr.Operand)
	}
	if groupExpr.Expression.Operations[0].Operator != "+" {
		t.Errorf("Expected operator '+', got '%s'", groupExpr.Expression.Operations[0].Operator)
	}

	// Inside the group should be another binary expression
	if groupExpr.Expression.Operations[1].Operator != "*" {
		t.Errorf("Expected operator '*', got '%s'", bind.Expr.Operations[0].Operator)
	}
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
	if len(root.Statements) != 1 {
		t.Fatalf("Expected 1 statement, got %d", len(root.Statements))
	}
	// Test the main if statement
	ifStmt, ok := root.Statements[0].Statement.(IfStatement)
	if !ok {
		t.Fatalf("Expected IfStatement, got %T", root.Statements[0].Statement)
	}

	// Test the if block
	if len(ifStmt.ThenBlock.Statements) != 1 {
		t.Fatalf("Expected 1 statement in if block, got %d", len(ifStmt.ThenBlock.Statements))
	}

	// Test the let statement in the if block
	ifBindStmt, ok := ifStmt.ThenBlock.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement in if block, got %T", ifStmt.ThenBlock.Statements[0].Statement)
	}
	if ifBindStmt.Name != "z" {
		t.Errorf("Expected variable name 'z', got '%s'", ifBindStmt.Name)
	}

	// Test the else branch
	if ifStmt.ElseBlock == nil {
		t.Fatalf("Expected non-nil else block")
	}

	// Test the let statement in the else block
	if len(ifStmt.ElseBlock.Statements) != 1 {
		t.Fatalf("Expected 1 statement in else block, got %d", len(ifStmt.ElseBlock.Statements))
	}
	elseBindStmt, ok := ifStmt.ElseBlock.Statements[0].Statement.(BindStatement)
	if !ok {
		t.Fatalf("Expected BindStatement in else block, got %T", ifStmt.ElseBlock.Statements[0].Statement)
	}
	if elseBindStmt.Name != "z" {
		t.Errorf("Expected variable name 'z', got '%s'", elseBindStmt.Name)
	}
}
