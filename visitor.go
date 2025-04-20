// Package patina provides the implementation of the Patina language compiler.
package patina

import (
	"fmt"
	"reflect"
)

// ASTVisitor defines the visitor interface for traversing the Patina AST nodes.
// It follows the Visitor pattern to allow operations on the AST without modifying its structure.
type ASTVisitor interface {
	// Root node
	VisitRoot(node *Root) interface{}

	// Statement nodes
	VisitModule(node *Module) interface{}
	VisitFunctionStatement(node *FunctionStatement) interface{}
	VisitReturnStatement(node *ReturnStatement) interface{}
	VisitLetStatement(node *LetStatement) interface{}
	VisitBreakStatement(node *BreakStatement) interface{}
	VisitExpressionStatement(node *ExpressionStatement) interface{}

	// Expression nodes
	VisitIfExpression(node *IfExpression) interface{}
	VisitCaseExpression(node *CaseExpression) interface{}
	VisitLambda(node *Lambda) interface{}
	VisitStaticExpression(node *StaticExpression) interface{}
	VisitTypeExpression(node *TypeExpression) interface{}
	VisitBlock(node *Block) interface{}
	VisitOperation(node *Operation) interface{}

	// Atom nodes
	VisitGroupedExpression(node *GroupedExpression) interface{}
	VisitAtomicType(node *AtomicType) interface{}
	VisitAtomicValue(node *AtomicValue) interface{}
	VisitBinding(node *Binding) interface{}

	// Type nodes
	VisitPointerType(node *PointerType) interface{}
	VisitArrayType(node *ArrayType) interface{}
	VisitIntegerType(node *IntegerType) interface{}
	VisitFunctionType(node *FunctionType) interface{}

	// Value nodes
	VisitInteger(node *Integer) interface{}
	VisitString(node *String) interface{}

	// Other nodes
	VisitDecoratedStatement(node *DecoratedStatement) interface{}
	VisitDocComment(node *DocComment) interface{}
	VisitAnnotation(node *Annotation) interface{}
	VisitIfBranch(node *IfBranch) interface{}
	VisitCaseBranch(node *CaseBranch) interface{}
	VisitStringFragment(node *StringFragment) interface{}
	VisitFunctionParameter(node *FunctionParameter) interface{}
	VisitOperand(node *Operand) interface{}
	VisitPostfix(node *Postfix) interface{}
}

// NoopVisitor is a visitor that does nothing. It can be embedded in other visitors
// to provide default implementations for the ASTVisitor interface.
type NoopVisitor struct{}

func (*NoopVisitor) VisitRoot(node *Root) interface{}                           { return nil }
func (*NoopVisitor) VisitModule(node *Module) interface{}                         { return nil }
func (*NoopVisitor) VisitFunctionStatement(node *FunctionStatement) interface{}   { return nil }
func (*NoopVisitor) VisitReturnStatement(node *ReturnStatement) interface{}       { return nil }
func (*NoopVisitor) VisitLetStatement(node *LetStatement) interface{}             { return nil }
func (*NoopVisitor) VisitBreakStatement(node *BreakStatement) interface{}         { return nil }
func (*NoopVisitor) VisitExpressionStatement(node *ExpressionStatement) interface{} { return nil }
func (*NoopVisitor) VisitIfExpression(node *IfExpression) interface{}             { return nil }
func (*NoopVisitor) VisitCaseExpression(node *CaseExpression) interface{}         { return nil }
func (*NoopVisitor) VisitLambda(node *Lambda) interface{}                         { return nil }
func (*NoopVisitor) VisitStaticExpression(node *StaticExpression) interface{}     { return nil }
func (*NoopVisitor) VisitTypeExpression(node *TypeExpression) interface{}         { return nil }
func (*NoopVisitor) VisitBlock(node *Block) interface{}                           { return nil }
func (*NoopVisitor) VisitOperation(node *Operation) interface{}                   { return nil }
func (*NoopVisitor) VisitGroupedExpression(node *GroupedExpression) interface{}   { return nil }
func (*NoopVisitor) VisitAtomicType(node *AtomicType) interface{}                 { return nil }
func (*NoopVisitor) VisitAtomicValue(node *AtomicValue) interface{}               { return nil }
func (*NoopVisitor) VisitBinding(node *Binding) interface{}                       { return nil }
func (*NoopVisitor) VisitPointerType(node *PointerType) interface{}               { return nil }
func (*NoopVisitor) VisitArrayType(node *ArrayType) interface{}                   { return nil }
func (*NoopVisitor) VisitIntegerType(node *IntegerType) interface{}               { return nil }
func (*NoopVisitor) VisitFunctionType(node *FunctionType) interface{}             { return nil }
func (*NoopVisitor) VisitInteger(node *Integer) interface{}                       { return nil }
func (*NoopVisitor) VisitString(node *String) interface{}                         { return nil }
func (*NoopVisitor) VisitDecoratedStatement(node *DecoratedStatement) interface{} { return nil }
func (*NoopVisitor) VisitDocComment(node *DocComment) interface{}                 { return nil }
func (*NoopVisitor) VisitAnnotation(node *Annotation) interface{}                 { return nil }
func (*NoopVisitor) VisitIfBranch(node *IfBranch) interface{}                     { return nil }
func (*NoopVisitor) VisitCaseBranch(node *CaseBranch) interface{}                 { return nil }
func (*NoopVisitor) VisitStringFragment(node *StringFragment) interface{}         { return nil }
func (*NoopVisitor) VisitFunctionParameter(node *FunctionParameter) interface{}   { return nil }
func (*NoopVisitor) VisitOperand(node *Operand) interface{}                       { return nil }
func (*NoopVisitor) VisitPostfix(node *Postfix) interface{}                       { return nil }

// Visit applies the appropriate visitor method to an AST node based on its type.
// This is a helper function that dispatches to the correct visitor method.
func Visit(visitor ASTVisitor, node interface{}) interface{} {
	// Skip nil nodes
	if node == nil {
		fmt.Println("Visit: Received nil node")
		return nil
	}

	// Debug log
	fmt.Printf("Visit: Received node type: %T\n", node)

	switch n := node.(type) {
	// Root
	case *Root:
		fmt.Println("Visit: Matched *Root")
		return visitor.VisitRoot(n)

	// Statements
	case *Module:
		fmt.Println("Visit: Matched *Module")
		return visitor.VisitModule(n)
	case *FunctionStatement:
		fmt.Println("Visit: Matched *FunctionStatement")
		return visitor.VisitFunctionStatement(n)
	case *ReturnStatement:
		fmt.Println("Visit: Matched *ReturnStatement")
		return visitor.VisitReturnStatement(n)
	case *LetStatement:
		fmt.Println("Visit: Matched *LetStatement")
		return visitor.VisitLetStatement(n)
	case *BreakStatement:
		fmt.Println("Visit: Matched *BreakStatement")
		return visitor.VisitBreakStatement(n)
	case *ExpressionStatement:
		fmt.Println("Visit: Matched *ExpressionStatement")
		return visitor.VisitExpressionStatement(n)

	// Handle the Statement interface type
	case Statement:
		fmt.Printf("Visit: Matched Statement interface (concrete type: %T)\n", n)
		// Dispatch based on the concrete type
		switch s := n.(type) {
		case *Module:
			fmt.Println("Visit: Statement dispatch -> *Module")
			return visitor.VisitModule(s)
		case *FunctionStatement:
			fmt.Println("Visit: Statement dispatch -> *FunctionStatement")
			return visitor.VisitFunctionStatement(s)
		case *ReturnStatement:
			fmt.Println("Visit: Statement dispatch -> *ReturnStatement")
			return visitor.VisitReturnStatement(s)
		case *LetStatement:
			fmt.Println("Visit: Statement dispatch -> *LetStatement")
			return visitor.VisitLetStatement(s)
		case *BreakStatement:
			fmt.Println("Visit: Statement dispatch -> *BreakStatement")
			return visitor.VisitBreakStatement(s)
		case *ExpressionStatement:
			fmt.Println("Visit: Statement dispatch -> *ExpressionStatement")
			return visitor.VisitExpressionStatement(s)
		default:
			fmt.Printf("Visit: Statement dispatch -> UNHANDLED concrete type: %T\n", s)
		}

	// Expressions
	case *IfExpression:
		fmt.Println("Visit: Matched *IfExpression")
		return visitor.VisitIfExpression(n)
	case *CaseExpression:
		fmt.Println("Visit: Matched *CaseExpression")
		return visitor.VisitCaseExpression(n)
	case *Lambda:
		fmt.Println("Visit: Matched *Lambda")
		return visitor.VisitLambda(n)
	case *StaticExpression:
		fmt.Println("Visit: Matched *StaticExpression")
		return visitor.VisitStaticExpression(n)
	case *TypeExpression:
		fmt.Println("Visit: Matched *TypeExpression")
		return visitor.VisitTypeExpression(n)
	case *Block:
		fmt.Println("Visit: Matched *Block")
		return visitor.VisitBlock(n)
	case *Operation:
		fmt.Println("Visit: Matched *Operation")
		return visitor.VisitOperation(n)

	// Handle the Expression interface type
	case Expression:
		fmt.Printf("Visit: Matched Expression interface (concrete type: %T)\n", n)
		// Dispatch based on the concrete type
		switch e := n.(type) {
		case *IfExpression:
			fmt.Println("Visit: Expression dispatch -> *IfExpression")
			return visitor.VisitIfExpression(e)
		case *CaseExpression:
			fmt.Println("Visit: Expression dispatch -> *CaseExpression")
			return visitor.VisitCaseExpression(e)
		case *Lambda:
			fmt.Println("Visit: Expression dispatch -> *Lambda")
			return visitor.VisitLambda(e)
		case *StaticExpression:
			fmt.Println("Visit: Expression dispatch -> *StaticExpression")
			return visitor.VisitStaticExpression(e)
		case *TypeExpression:
			fmt.Println("Visit: Expression dispatch -> *TypeExpression")
			return visitor.VisitTypeExpression(e)
		case *Block:
			fmt.Println("Visit: Expression dispatch -> *Block")
			return visitor.VisitBlock(e)
		case *Operation:
			fmt.Println("Visit: Expression dispatch -> *Operation")
			return visitor.VisitOperation(e)
		default:
			fmt.Printf("Visit: Expression dispatch -> UNHANDLED concrete type: %T\n", e)
		}

	// Atoms
	case *GroupedExpression:
		fmt.Println("Visit: Matched *GroupedExpression")
		return visitor.VisitGroupedExpression(n)
	case *AtomicType:
		fmt.Println("Visit: Matched *AtomicType")
		return visitor.VisitAtomicType(n)
	case *AtomicValue:
		fmt.Println("Visit: Matched *AtomicValue")
		return visitor.VisitAtomicValue(n)
	case *Binding:
		fmt.Println("Visit: Matched *Binding")
		return visitor.VisitBinding(n)

	// Handle the Atom interface type
	case Atom:
		fmt.Printf("Visit: Matched Atom interface (concrete type: %T)\n", n)
		// Dispatch based on the concrete type
		switch a := n.(type) {
		case *GroupedExpression:
			fmt.Println("Visit: Atom dispatch -> *GroupedExpression")
			return visitor.VisitGroupedExpression(a)
		case *AtomicType:
			fmt.Println("Visit: Atom dispatch -> *AtomicType")
			return visitor.VisitAtomicType(a)
		case *AtomicValue:
			fmt.Println("Visit: Atom dispatch -> *AtomicValue")
			return visitor.VisitAtomicValue(a)
		case *Binding:
			fmt.Println("Visit: Atom dispatch -> *Binding")
			return visitor.VisitBinding(a)
		default:
			fmt.Printf("Visit: Atom dispatch -> UNHANDLED concrete type: %T\n", a)
		}

	// Types
	case *PointerType:
		fmt.Println("Visit: Matched *PointerType")
		return visitor.VisitPointerType(n)
	case *ArrayType:
		fmt.Println("Visit: Matched *ArrayType")
		return visitor.VisitArrayType(n)
	case *IntegerType:
		fmt.Println("Visit: Matched *IntegerType")
		return visitor.VisitIntegerType(n)
	case *FunctionType:
		fmt.Println("Visit: Matched *FunctionType")
		return visitor.VisitFunctionType(n)

	// Handle the Type interface type
	case Type:
		fmt.Printf("Visit: Matched Type interface (concrete type: %T)\n", n)
		// Dispatch based on the concrete type
		switch t := n.(type) {
		case *PointerType:
			fmt.Println("Visit: Type dispatch -> *PointerType")
			return visitor.VisitPointerType(t)
		case *ArrayType:
			fmt.Println("Visit: Type dispatch -> *ArrayType")
			return visitor.VisitArrayType(t)
		case *IntegerType:
			fmt.Println("Visit: Type dispatch -> *IntegerType")
			return visitor.VisitIntegerType(t)
		case *FunctionType:
			fmt.Println("Visit: Type dispatch -> *FunctionType")
			return visitor.VisitFunctionType(t)
		default:
			fmt.Printf("Visit: Type dispatch -> UNHANDLED concrete type: %T\n", t)
		}

	// Values
	case *Integer:
		fmt.Println("Visit: Matched *Integer")
		return visitor.VisitInteger(n)
	case *String:
		fmt.Println("Visit: Matched *String")
		return visitor.VisitString(n)

	// Handle the Value interface type
	case Value:
		fmt.Printf("Visit: Matched Value interface (concrete type: %T)\n", n)
		// Dispatch based on the concrete type
		switch v := n.(type) {
		case *Integer:
			fmt.Println("Visit: Value dispatch -> *Integer")
			return visitor.VisitInteger(v)
		case *String:
			fmt.Println("Visit: Value dispatch -> *String")
			return visitor.VisitString(v)
		default:
			fmt.Printf("Visit: Value dispatch -> UNHANDLED concrete type: %T\n", v)
		}

	// Other nodes
	case *DecoratedStatement:
		fmt.Println("Visit: Matched *DecoratedStatement")
		return visitor.VisitDecoratedStatement(n)
	case *DocComment:
		fmt.Println("Visit: Matched *DocComment")
		return visitor.VisitDocComment(n)
	case *Annotation:
		fmt.Println("Visit: Matched *Annotation")
		return visitor.VisitAnnotation(n)
	case *IfBranch:
		fmt.Println("Visit: Matched *IfBranch")
		return visitor.VisitIfBranch(n)
	case *CaseBranch:
		fmt.Println("Visit: Matched *CaseBranch")
		return visitor.VisitCaseBranch(n)
	case *StringFragment:
		fmt.Println("Visit: Matched *StringFragment")
		return visitor.VisitStringFragment(n)
	case *FunctionParameter:
		fmt.Println("Visit: Matched *FunctionParameter")
		return visitor.VisitFunctionParameter(n)
	case *Operand:
		fmt.Println("Visit: Matched *Operand")
		return visitor.VisitOperand(n)
	case *Postfix:
		fmt.Println("Visit: Matched *Postfix")
		return visitor.VisitPostfix(n)
	default:
		// Use reflect to get the type name for unhandled types
		typeName := reflect.TypeOf(node).String()
		fmt.Printf("Visit: UNHANDLED node type: %s\n", typeName)
	}

	return nil
}

// RecursiveVisitor processes all nodes in the AST recursively.
// It wraps another ASTVisitor and delegates to that visitor's methods,
// but also ensures all child nodes are visited.
type RecursiveVisitor struct {
	Delegate ASTVisitor
}

func (v *RecursiveVisitor) VisitRoot(node *Root) interface{} {
	result := v.Delegate.VisitRoot(node)
	for i := range node.Statements {
		Visit(v, &node.Statements[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitModule(node *Module) interface{} {
	result := v.Delegate.VisitModule(node)
	Visit(v, &node.Body)
	return result
}

func (v *RecursiveVisitor) VisitFunctionStatement(node *FunctionStatement) interface{} {
	result := v.Delegate.VisitFunctionStatement(node)
	for i := range node.Parameters {
		Visit(v, &node.Parameters[i])
	}
	if node.Return != nil {
		Visit(v, node.Return)
	}
	Visit(v, &node.Block)
	return result
}

func (v *RecursiveVisitor) VisitReturnStatement(node *ReturnStatement) interface{} {
	result := v.Delegate.VisitReturnStatement(node)
	if node.Value != nil {
		Visit(v, node.Value)
	}
	return result
}

func (v *RecursiveVisitor) VisitLetStatement(node *LetStatement) interface{} {
	result := v.Delegate.VisitLetStatement(node)
	if node.Type != nil {
		Visit(v, node.Type)
	}
	if node.Value != nil {
		Visit(v, node.Value)
	}
	return result
}

func (v *RecursiveVisitor) VisitBreakStatement(node *BreakStatement) interface{} {
	result := v.Delegate.VisitBreakStatement(node)
	if node.Value != nil {
		Visit(v, node.Value)
	}
	return result
}

func (v *RecursiveVisitor) VisitExpressionStatement(node *ExpressionStatement) interface{} {
	result := v.Delegate.VisitExpressionStatement(node)
	Visit(v, node.Expression)
	return result
}

func (v *RecursiveVisitor) VisitIfExpression(node *IfExpression) interface{} {
	result := v.Delegate.VisitIfExpression(node)
	Visit(v, &node.If)
	for i := range node.ElseIfs {
		Visit(v, &node.ElseIfs[i])
	}
	if node.Else != nil {
		Visit(v, node.Else)
	}
	return result
}

func (v *RecursiveVisitor) VisitCaseExpression(node *CaseExpression) interface{} {
	result := v.Delegate.VisitCaseExpression(node)
	for i := range node.Branches {
		Visit(v, &node.Branches[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitLambda(node *Lambda) interface{} {
	result := v.Delegate.VisitLambda(node)
	for i := range node.Parameters {
		Visit(v, &node.Parameters[i])
	}
	if node.Return != nil {
		Visit(v, node.Return)
	}
	Visit(v, &node.Body)
	return result
}

func (v *RecursiveVisitor) VisitStaticExpression(node *StaticExpression) interface{} {
	result := v.Delegate.VisitStaticExpression(node)
	Visit(v, node.Static)
	return result
}

func (v *RecursiveVisitor) VisitTypeExpression(node *TypeExpression) interface{} {
	result := v.Delegate.VisitTypeExpression(node)
	// Types are a union type, can't use Visit directly
	switch t := node.Type.(type) {
	case *IntegerType:
		Visit(v, t)
	case *PointerType:
		Visit(v, t)
	case *ArrayType:
		Visit(v, t)
	case *FunctionType:
		Visit(v, t)
	}
	return result
}

func (v *RecursiveVisitor) VisitBlock(node *Block) interface{} {
	result := v.Delegate.VisitBlock(node)
	for i := range node.Statements {
		Visit(v, &node.Statements[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitOperation(node *Operation) interface{} {
	result := v.Delegate.VisitOperation(node)
	for i := range node.Operands {
		Visit(v, &node.Operands[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitGroupedExpression(node *GroupedExpression) interface{} {
	result := v.Delegate.VisitGroupedExpression(node)
	Visit(v, node.Expression)
	return result
}

func (v *RecursiveVisitor) VisitAtomicType(node *AtomicType) interface{} {
	result := v.Delegate.VisitAtomicType(node)
	// Types are a union type, can't use Visit directly
	switch t := node.Type.(type) {
	case *IntegerType:
		Visit(v, t)
	case *PointerType:
		Visit(v, t)
	case *ArrayType:
		Visit(v, t)
	case *FunctionType:
		Visit(v, t)
	}
	return result
}

func (v *RecursiveVisitor) VisitAtomicValue(node *AtomicValue) interface{} {
	result := v.Delegate.VisitAtomicValue(node)
	switch val := node.Value.(type) {
	case *Integer:
		Visit(v, val)
	case *String:
		Visit(v, val)
	}
	return result
}

func (v *RecursiveVisitor) VisitBinding(node *Binding) interface{} {
	return v.Delegate.VisitBinding(node)
}

func (v *RecursiveVisitor) VisitPointerType(node *PointerType) interface{} {
	result := v.Delegate.VisitPointerType(node)
	Visit(v, node.Target)
	return result
}

func (v *RecursiveVisitor) VisitArrayType(node *ArrayType) interface{} {
	result := v.Delegate.VisitArrayType(node)
	Visit(v, node.Element)
	return result
}

func (v *RecursiveVisitor) VisitIntegerType(node *IntegerType) interface{} {
	return v.Delegate.VisitIntegerType(node)
}

func (v *RecursiveVisitor) VisitFunctionType(node *FunctionType) interface{} {
	result := v.Delegate.VisitFunctionType(node)
	for i := range node.Parameters {
		Visit(v, &node.Parameters[i])
	}
	if node.Return != nil {
		Visit(v, node.Return)
	}
	return result
}

func (v *RecursiveVisitor) VisitInteger(node *Integer) interface{} {
	return v.Delegate.VisitInteger(node)
}

func (v *RecursiveVisitor) VisitString(node *String) interface{} {
	result := v.Delegate.VisitString(node)
	for i := range node.Fragments {
		Visit(v, &node.Fragments[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitDecoratedStatement(node *DecoratedStatement) interface{} {
	result := v.Delegate.VisitDecoratedStatement(node)
	if node.Comments != nil {
		Visit(v, node.Comments)
	}
	for i := range node.Annotations {
		Visit(v, &node.Annotations[i])
	}

	// Visit the actual statement directly using explicit type checking
	switch stmt := node.Statement.(type) {
	case *Module:
		Visit(v, stmt)
	case *FunctionStatement:
		Visit(v, stmt)
	case *ReturnStatement:
		Visit(v, stmt)
	case *LetStatement:
		Visit(v, stmt)
	case *BreakStatement:
		Visit(v, stmt)
	case *ExpressionStatement:
		Visit(v, stmt)
	}
	return result
}

func (v *RecursiveVisitor) VisitDocComment(node *DocComment) interface{} {
	result := v.Delegate.VisitDocComment(node)
	for i := range node.Fragments {
		if node.Fragments[i].Comment != nil {
			Visit(v, node.Fragments[i].Comment)
		}
	}
	return result
}

func (v *RecursiveVisitor) VisitAnnotation(node *Annotation) interface{} {
	result := v.Delegate.VisitAnnotation(node)
	for i := range node.Arguments {
		Visit(v, node.Arguments[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitIfBranch(node *IfBranch) interface{} {
	result := v.Delegate.VisitIfBranch(node)
	Visit(v, node.Condition)
	Visit(v, node.Then)
	return result
}

func (v *RecursiveVisitor) VisitCaseBranch(node *CaseBranch) interface{} {
	result := v.Delegate.VisitCaseBranch(node)
	if node.If != nil {
		Visit(v, node.If)
	}
	if node.Else != nil {
		Visit(v, node.Else)
	}
	return result
}

func (v *RecursiveVisitor) VisitStringFragment(node *StringFragment) interface{} {
	result := v.Delegate.VisitStringFragment(node)
	if node.Interpolation != nil {
		Visit(v, node.Interpolation)
	}
	return result
}

func (v *RecursiveVisitor) VisitFunctionParameter(node *FunctionParameter) interface{} {
	result := v.Delegate.VisitFunctionParameter(node)
	Visit(v, node.Type)
	return result
}

func (v *RecursiveVisitor) VisitOperand(node *Operand) interface{} {
	result := v.Delegate.VisitOperand(node)
	Visit(v, node.Atom)
	for i := range node.Postfix {
		Visit(v, &node.Postfix[i])
	}
	return result
}

func (v *RecursiveVisitor) VisitPostfix(node *Postfix) interface{} {
	result := v.Delegate.VisitPostfix(node)
	for i := range node.Invocation {
		Visit(v, node.Invocation[i])
	}
	if node.Index != nil {
		Visit(v, node.Index)
	}
	return result
}
