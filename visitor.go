// visitor.go
package patina

// ASTVisitor defines the interface for visiting each node type in the AST.
type ASTVisitor interface {
	VisitRoot(node *Root) interface{}
	VisitModule(node *Module) interface{}
	VisitFunctionStatement(node *FunctionStatement) interface{}
	VisitReturnStatement(node *ReturnStatement) interface{}
	VisitLetStatement(node *LetStatement) interface{}
	VisitBreakStatement(node *BreakStatement) interface{}
	VisitExpressionStatement(node *ExpressionStatement) interface{}
	VisitIfExpression(node *IfExpression) interface{}
	VisitCaseExpression(node *CaseExpression) interface{}
	VisitLambda(node *Lambda) interface{}
	VisitStaticExpression(node *StaticExpression) interface{}
	VisitTypeExpression(node *TypeExpression) interface{}
	VisitBlock(node *Block) interface{}
	VisitOperation(node *Operation) interface{}
	VisitGroupedExpression(node *GroupedExpression) interface{}
	VisitAtomicType(node *AtomicType) interface{}
	VisitAtomicValue(node *AtomicValue) interface{}
	VisitBinding(node *Binding) interface{}
	VisitPointerType(node *PointerType) interface{}
	VisitArrayType(node *ArrayType) interface{}
	VisitIntegerType(node *IntegerType) interface{}
	VisitFunctionType(node *FunctionType) interface{}
	VisitInteger(node *Integer) interface{}
	VisitString(node *String) interface{}
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

// Visit dispatches the appropriate visitor method based on the node's concrete type.
func Visit(visitor ASTVisitor, node interface{}) interface{} {
	if node == nil {
		return nil
	}

	// Handle specific concrete types
	switch n := node.(type) {
	case *Root:
		return visitor.VisitRoot(n)
	case *Module:
		return visitor.VisitModule(n)
	case *FunctionStatement:
		return visitor.VisitFunctionStatement(n)
	case *ReturnStatement:
		return visitor.VisitReturnStatement(n)
	case *LetStatement:
		return visitor.VisitLetStatement(n)
	case *BreakStatement:
		return visitor.VisitBreakStatement(n)
	case *ExpressionStatement:
		return visitor.VisitExpressionStatement(n)
	case *IfExpression:
		return visitor.VisitIfExpression(n)
	case *CaseExpression:
		return visitor.VisitCaseExpression(n)
	case *Lambda:
		return visitor.VisitLambda(n)
	case *StaticExpression:
		return visitor.VisitStaticExpression(n)
	case *TypeExpression:
		return visitor.VisitTypeExpression(n)
	case *Block:
		return visitor.VisitBlock(n)
	case *Operation:
		return visitor.VisitOperation(n)
	case *GroupedExpression:
		return visitor.VisitGroupedExpression(n)
	case *AtomicType:
		return visitor.VisitAtomicType(n)
	case *AtomicValue:
		return visitor.VisitAtomicValue(n)
	case *Binding:
		return visitor.VisitBinding(n)
	case *PointerType:
		return visitor.VisitPointerType(n)
	case *ArrayType:
		return visitor.VisitArrayType(n)
	case *IntegerType:
		return visitor.VisitIntegerType(n)
	case *FunctionType:
		return visitor.VisitFunctionType(n)
	case *Integer:
		return visitor.VisitInteger(n)
	case *String:
		return visitor.VisitString(n)
	case *DecoratedStatement:
		return visitor.VisitDecoratedStatement(n)
	case *DocComment:
		return visitor.VisitDocComment(n)
	case *Annotation:
		return visitor.VisitAnnotation(n)
	case *IfBranch:
		return visitor.VisitIfBranch(n)
	case *CaseBranch:
		return visitor.VisitCaseBranch(n)
	case *StringFragment:
		return visitor.VisitStringFragment(n)
	case *FunctionParameter:
		return visitor.VisitFunctionParameter(n)
	case *Operand:
		return visitor.VisitOperand(n)
	case *Postfix:
		return visitor.VisitPostfix(n)

	// Handle interface types by unwrapping if possible
	// Note: This might be less robust than explicit Accept methods, but avoids parser changes.
	case Statement:
		return Visit(visitor, UnwrapStatement(n)) // Need an Unwrap helper
	case Expression:
		return Visit(visitor, UnwrapExpression(n))
	case Atom:
		return Visit(visitor, UnwrapAtom(n))
	case Type:
		return Visit(visitor, UnwrapType(n))
	case Value:
		return Visit(visitor, UnwrapValue(n))
	}

	// Consider panicking here for unhandled types if Unwrap helpers don't cover all cases.
	// fmt.Printf("Visit: Unhandled node type %T\n", node)
	return nil
}

// --- Unwrap Helpers (To handle interface types in Visit) ---
// These are needed because DecoratedStatement.Statement is of type Statement (interface)

func UnwrapStatement(stmt Statement) interface{} {
	switch s := stmt.(type) {
	case *Module:
		return s
	case *FunctionStatement:
		return s
	case *ReturnStatement:
		return s
	case *LetStatement:
		return s
	case *BreakStatement:
		return s
	case *ExpressionStatement:
		return s
	default:
		return nil // Or panic
	}
}

func UnwrapExpression(expr Expression) interface{} {
	switch e := expr.(type) {
	case *IfExpression:
		return e
	case *CaseExpression:
		return e
	case *Lambda:
		return e
	case *StaticExpression:
		return e
	case *TypeExpression:
		return e
	case *Block:
		return e
	case *Operation:
		return e
	default:
		return nil // Or panic
	}
}

func UnwrapAtom(atom Atom) interface{} {
	switch a := atom.(type) {
	case *GroupedExpression:
		return a
	case *AtomicType:
		return a
	case *AtomicValue:
		return a
	case *Binding:
		return a
	default:
		return nil // Or panic
	}
}

func UnwrapType(t Type) interface{} {
	switch ty := t.(type) {
	case *PointerType:
		return ty
	case *ArrayType:
		return ty
	case *IntegerType:
		return ty
	case *FunctionType:
		return ty
	default:
		return nil // Or panic
	}
}

func UnwrapValue(v Value) interface{} {
	switch va := v.(type) {
	case *Integer:
		return va
	case *String:
		return va
	default:
		return nil // Or panic
	}
}

// RecursiveVisitor traverses the entire AST, delegating actions to its Delegate.
type RecursiveVisitor struct {
	Delegate ASTVisitor
}

// --- RecursiveVisitor Methods ---

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
	Visit(v, node.Type) // Visit the underlying Type
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
	Visit(v, node.Type) // Visit the underlying Type
	return result
}
func (v *RecursiveVisitor) VisitAtomicValue(node *AtomicValue) interface{} {
	result := v.Delegate.VisitAtomicValue(node)
	Visit(v, node.Value) // Visit the underlying Value
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
	// Delegate FIRST for the decoration itself
	result := v.Delegate.VisitDecoratedStatement(node)
	// Then visit children recursively
	if node.Comments != nil {
		Visit(v, node.Comments)
	}
	for i := range node.Annotations {
		Visit(v, &node.Annotations[i])
	}
	if node.Statement != nil {
		Visit(v, node.Statement)
	} // Visit underlying statement
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
