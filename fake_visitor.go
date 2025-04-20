package patina

import (
	"strings"
)

// FakeVisitor is an implementation of ASTVisitor for testing purposes.
// It records which nodes were visited and in what order.
type FakeVisitor struct {
	// VisitLog contains the names of all visited nodes in order
	VisitLog []string

	// NodeCount tracks the number of visits per node type
	NodeCount map[string]int

	// ExtractedValues can be used to store specific values extracted during traversal
	ExtractedValues map[string]interface{}
}

// NewFakeVisitor creates a new FakeVisitor with initialized fields
func NewFakeVisitor() *FakeVisitor {
	return &FakeVisitor{
		VisitLog:        make([]string, 0),
		NodeCount:       make(map[string]int),
		ExtractedValues: make(map[string]interface{}),
	}
}

// logVisit records a visit to a specific node type
func (v *FakeVisitor) logVisit(nodeType string) {
	v.VisitLog = append(v.VisitLog, nodeType)
	v.NodeCount[nodeType]++
}

// VisitRoot records a visit to the Root node
func (v *FakeVisitor) VisitRoot(node *Root) interface{} {
	v.logVisit("Root")
	for i := range node.Statements {
		Visit(v, &node.Statements[i]) // Pass address of slice element
	}
	return nil
}

// VisitModule records a visit to the Module node
func (v *FakeVisitor) VisitModule(node *Module) interface{} {
	v.logVisit("Module")
	v.ExtractedValues["ModuleName"] = node.Name
	Visit(v, &node.Body)
	return nil
}

// VisitFunctionStatement records a visit to the FunctionStatement node
func (v *FakeVisitor) VisitFunctionStatement(node *FunctionStatement) interface{} {
	v.logVisit("FunctionStatement")
	v.ExtractedValues["FunctionName"] = node.Name

	// Visit parameters
	for _, param := range node.Parameters {
		Visit(v, &param)
	}

	// Visit return type if present
	if node.Return != nil {
		Visit(v, node.Return)
	}

	// Visit function body
	Visit(v, &node.Block)
	return nil
}

// VisitReturnStatement records a visit to the ReturnStatement node
func (v *FakeVisitor) VisitReturnStatement(node *ReturnStatement) interface{} {
	v.logVisit("ReturnStatement")
	if node.Value != nil {
		Visit(v, node.Value)
	}
	return nil
}

// VisitLetStatement records a visit to the LetStatement node
func (v *FakeVisitor) VisitLetStatement(node *LetStatement) interface{} {
	v.logVisit("LetStatement")
	v.ExtractedValues["LetName"] = node.Name

	if node.Type != nil {
		Visit(v, node.Type)
	}

	if node.Value != nil {
		Visit(v, node.Value)
	}
	return nil
}

// VisitBreakStatement records a visit to the BreakStatement node
func (v *FakeVisitor) VisitBreakStatement(node *BreakStatement) interface{} {
	v.logVisit("BreakStatement")
	if node.Value != nil {
		Visit(v, node.Value)
	}
	return nil
}

// VisitExpressionStatement records a visit to the ExpressionStatement node
func (v *FakeVisitor) VisitExpressionStatement(node *ExpressionStatement) interface{} {
	v.logVisit("ExpressionStatement")
	Visit(v, node.Expression)
	return nil
}

// VisitIfExpression records a visit to the IfExpression node
func (v *FakeVisitor) VisitIfExpression(node *IfExpression) interface{} {
	v.logVisit("IfExpression")
	Visit(v, &node.If)

	for _, elseIf := range node.ElseIfs {
		Visit(v, &elseIf)
	}

	if node.Else != nil {
		Visit(v, node.Else)
	}
	return nil
}

// VisitCaseExpression records a visit to the CaseExpression node
func (v *FakeVisitor) VisitCaseExpression(node *CaseExpression) interface{} {
	v.logVisit("CaseExpression")
	for _, branch := range node.Branches {
		Visit(v, &branch)
	}
	return nil
}

// VisitLambda records a visit to the Lambda node
func (v *FakeVisitor) VisitLambda(node *Lambda) interface{} {
	v.logVisit("Lambda")

	// Visit parameters
	for _, param := range node.Parameters {
		Visit(v, &param)
	}

	// Visit return type if present
	if node.Return != nil {
		Visit(v, node.Return)
	}

	// Visit lambda body
	Visit(v, &node.Body)
	return nil
}

// VisitStaticExpression records a visit to the StaticExpression node
func (v *FakeVisitor) VisitStaticExpression(node *StaticExpression) interface{} {
	v.logVisit("StaticExpression")
	Visit(v, node.Static)
	return nil
}

// VisitTypeExpression records a visit to the TypeExpression node
func (v *FakeVisitor) VisitTypeExpression(node *TypeExpression) interface{} {
	v.logVisit("TypeExpression")
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
	return nil
}

// VisitBlock records a visit to the Block node
func (v *FakeVisitor) VisitBlock(node *Block) interface{} {
	v.logVisit("Block")
	if node.Label != "" {
		v.ExtractedValues["BlockLabel"] = node.Label
	}

	for _, stmt := range node.Statements {
		Visit(v, &stmt)
	}
	return nil
}

// VisitOperation records a visit to the Operation node
func (v *FakeVisitor) VisitOperation(node *Operation) interface{} {
	v.logVisit("Operation")
	if len(node.Prefix) > 0 {
		v.ExtractedValues["PrefixOperator"] = strings.Join(node.Prefix, ",")
	}
	for i := range node.Operands {
		Visit(v, &node.Operands[i]) // Visit each operand
	}
	return nil
}

// VisitGroupedExpression records a visit to the GroupedExpression node
func (v *FakeVisitor) VisitGroupedExpression(node *GroupedExpression) interface{} {
	v.logVisit("GroupedExpression")
	Visit(v, node.Expression)
	return nil
}

// VisitAtomicType records a visit to the AtomicType node
func (v *FakeVisitor) VisitAtomicType(node *AtomicType) interface{} {
	v.logVisit("AtomicType")
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
	return nil
}

// VisitAtomicValue records a visit to the AtomicValue node
func (v *FakeVisitor) VisitAtomicValue(node *AtomicValue) interface{} {
	v.logVisit("AtomicValue")
	// Values are a union type, can't use Visit directly
	switch val := node.Value.(type) {
	case *Integer:
		Visit(v, val)
	case *String:
		Visit(v, val)
	}
	return nil
}

// VisitBinding records a visit to the Binding node
func (v *FakeVisitor) VisitBinding(node *Binding) interface{} {
	v.logVisit("Binding")
	// Binding only contains Ident string, no expression to visit
	return nil
}

// VisitPointerType records a visit to the PointerType node
func (v *FakeVisitor) VisitPointerType(node *PointerType) interface{} {
	v.logVisit("PointerType")
	Visit(v, node.Target) // Visit the target Atom
	return nil
}

// VisitArrayType records a visit to the ArrayType node
func (v *FakeVisitor) VisitArrayType(node *ArrayType) interface{} {
	v.logVisit("ArrayType")
	if node.Dimension != nil {
		// Potentially extract dimension value if needed by tests
		// v.ExtractedValues["ArrayDimension"] = *node.Dimension
	}
	Visit(v, node.Element) // Visit the element Atom
	return nil
}

// VisitIntegerType records a visit to the IntegerType node
func (v *FakeVisitor) VisitIntegerType(node *IntegerType) interface{} {
	v.logVisit("IntegerType")
	v.ExtractedValues["IntegerSigned"] = node.Signed
	v.ExtractedValues["IntegerBitSize"] = node.BitSize
	return nil
}

// VisitFunctionType records a visit to the FunctionType node
func (v *FakeVisitor) VisitFunctionType(node *FunctionType) interface{} {
	v.logVisit("FunctionType")
	if node.Name != "" {
		v.ExtractedValues["FunctionTypeName"] = node.Name
	}

	for _, param := range node.Parameters {
		Visit(v, &param)
	}

	if node.Return != nil {
		Visit(v, node.Return)
	}
	return nil
}

// VisitInteger records a visit to the Integer node
func (v *FakeVisitor) VisitInteger(node *Integer) interface{} {
	v.logVisit("Integer")
	v.ExtractedValues["IntegerValue"] = node.Value
	return nil
}

// VisitString records a visit to the String node
func (v *FakeVisitor) VisitString(node *String) interface{} {
	v.logVisit("String")

	for _, fragment := range node.Fragments {
		Visit(v, &fragment)
	}
	return nil
}

// VisitDecoratedStatement records a visit to the DecoratedStatement node
func (v *FakeVisitor) VisitDecoratedStatement(node *DecoratedStatement) interface{} {
	v.logVisit("DecoratedStatement")

	// Visit Comments if present
	if node.Comments != nil {
		Visit(v, node.Comments)
	}

	// Visit Annotations
	for i := range node.Annotations {
		Visit(v, &node.Annotations[i])
	}

	// Visit the actual statement being decorated
	Visit(v, node.Statement)

	return nil
}

// VisitDocComment records a visit to the DocComment node
func (v *FakeVisitor) VisitDocComment(node *DocComment) interface{} {
	v.logVisit("DocComment")

	for _, fragment := range node.Fragments {
		if fragment.Text != "" {
			v.ExtractedValues["DocCommentText"] = fragment.Text
		}
		if fragment.Comment != nil {
			Visit(v, fragment.Comment)
		}
	}
	return nil
}

// VisitAnnotation records a visit to the Annotation node
func (v *FakeVisitor) VisitAnnotation(node *Annotation) interface{} {
	v.logVisit("Annotation")
	v.ExtractedValues["AnnotationName"] = node.Name // Name is a string

	for _, arg := range node.Arguments {
		Visit(v, arg) // Visit each argument Expression
	}
	return nil
}

// VisitIfBranch records a visit to the IfBranch node
func (v *FakeVisitor) VisitIfBranch(node *IfBranch) interface{} {
	v.logVisit("IfBranch")
	Visit(v, node.Condition)
	Visit(v, node.Then)
	return nil
}

// VisitCaseBranch records a visit to the CaseBranch node
func (v *FakeVisitor) VisitCaseBranch(node *CaseBranch) interface{} {
	v.logVisit("CaseBranch")

	if node.If != nil {
		Visit(v, node.If)
	}

	if node.Else != nil {
		Visit(v, node.Else)
	}
	return nil
}

// VisitStringFragment records a visit to the StringFragment node
func (v *FakeVisitor) VisitStringFragment(node *StringFragment) interface{} {
	v.logVisit("StringFragment")

	if node.Text != "" {
		v.ExtractedValues["StringFragmentText"] = node.Text
	}

	if node.Interpolation != nil {
		Visit(v, node.Interpolation)
	}
	return nil
}

// VisitFunctionParameter records a visit to the FunctionParameter node
func (v *FakeVisitor) VisitFunctionParameter(node *FunctionParameter) interface{} {
	v.logVisit("FunctionParameter")
	v.ExtractedValues["ParameterName"] = node.Name
	Visit(v, node.Type)
	return nil
}

// VisitOperand records a visit to the Operand node
func (v *FakeVisitor) VisitOperand(node *Operand) interface{} {
	v.logVisit("Operand")
	Visit(v, node.Atom) // Visit the core Atom

	// Visit relevant parts of postfix operations (calls, indexing)
	for i := range node.Postfix {
		postfix := &node.Postfix[i]
		if len(postfix.Invocation) > 0 {
			// This indicates a function call, visit arguments
			for _, arg := range postfix.Invocation {
				Visit(v, arg)
			}
		} else if postfix.Index != nil {
			// This indicates indexing, visit the index expression
			Visit(v, postfix.Index)
		}
		// postfix.Operator is just a string, no need to visit
	}
	return nil
}

// VisitPostfix records a visit to the Postfix node
func (v *FakeVisitor) VisitPostfix(node *Postfix) interface{} {
	v.logVisit("Postfix")

	for _, arg := range node.Invocation {
		Visit(v, arg)
	}

	if node.Index != nil {
		Visit(v, node.Index)
	}

	if node.Operator != "" {
		v.ExtractedValues["PostfixOperator"] = node.Operator
	}
	return nil
}
