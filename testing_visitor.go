// testing_visitor.go
package patina

import "reflect"

// TestingVisitorResult holds the data collected by TestingVisitor for snapshotting.
type TestingVisitorResult struct {
	VisitLog        []string               `json:"visitLog"`
	NodeCount       map[string]int         `json:"nodeCount"`
	ExtractedValues map[string]interface{} `json:"extractedValues"`
}

// TestingVisitor implements ASTVisitor for testing.
// It is non-recursive and relies on an external driver (like RecursiveVisitor).
// It records visited nodes and extracts basic values for snapshot testing.
type TestingVisitor struct {
	NoopVisitor // Embed NoopVisitor to avoid implementing all methods initially

	VisitLog        []string
	NodeCount       map[string]int
	ExtractedValues map[string]interface{}
}

// NewTestingVisitor creates a new TestingVisitor.
func NewTestingVisitor() *TestingVisitor {
	return &TestingVisitor{
		VisitLog:        make([]string, 0),
		NodeCount:       make(map[string]int),
		ExtractedValues: make(map[string]interface{}),
	}
}

// logVisit records the visit and increments counts.
func (v *TestingVisitor) logVisit(node interface{}) {
	// Use reflection to get the type name without the package prefix
	nodeType := reflect.TypeOf(node).Elem().Name()
	v.VisitLog = append(v.VisitLog, nodeType)
	v.NodeCount[nodeType]++
}

// Result returns the collected data for snapshotting.
func (v *TestingVisitor) Result() *TestingVisitorResult {
	return &TestingVisitorResult{
		VisitLog:        v.VisitLog,
		NodeCount:       v.NodeCount,
		ExtractedValues: v.ExtractedValues,
	}
}

// Implement Visit methods for relevant nodes to log/extract data

func (v *TestingVisitor) VisitRoot(node *Root) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitModule(node *Module) interface{} {
	v.logVisit(node)
	v.ExtractedValues["ModuleName"] = node.Name
	return nil
}
func (v *TestingVisitor) VisitFunctionStatement(node *FunctionStatement) interface{} {
	v.logVisit(node)
	v.ExtractedValues["FunctionName"] = node.Name
	return nil
}
func (v *TestingVisitor) VisitReturnStatement(node *ReturnStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitLetStatement(node *LetStatement) interface{} {
	v.logVisit(node)
	v.ExtractedValues["LetName"] = node.Name
	return nil
}
func (v *TestingVisitor) VisitBreakStatement(node *BreakStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitExpressionStatement(node *ExpressionStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitIfExpression(node *IfExpression) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitCaseExpression(node *CaseExpression) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitLambda(node *Lambda) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitStaticExpression(node *StaticExpression) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitTypeExpression(node *TypeExpression) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitBlock(node *Block) interface{} {
	v.logVisit(node)
	if node.Label != "" {
		v.ExtractedValues["BlockLabel"] = node.Label
	}
	return nil
}
func (v *TestingVisitor) VisitOperation(node *Operation) interface{} {
	v.logVisit(node)
	if len(node.Prefix) > 0 {
		v.ExtractedValues["OperationPrefix"] = node.Prefix
	}
	return nil
}
func (v *TestingVisitor) VisitGroupedExpression(node *GroupedExpression) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitAtomicType(node *AtomicType) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitAtomicValue(node *AtomicValue) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitBinding(node *Binding) interface{} {
	v.logVisit(node)
	v.ExtractedValues["BindingName"] = node.Ident
	return nil
}
func (v *TestingVisitor) VisitPointerType(node *PointerType) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitArrayType(node *ArrayType) interface{} {
	v.logVisit(node)
	if node.Dimension != nil {
		v.ExtractedValues["ArrayDimension"] = *node.Dimension
	}
	return nil
}
func (v *TestingVisitor) VisitIntegerType(node *IntegerType) interface{} {
	v.logVisit(node)
	v.ExtractedValues["IntegerSigned"] = node.Signed
	v.ExtractedValues["IntegerBitSize"] = node.BitSize
	return nil
}
func (v *TestingVisitor) VisitFunctionType(node *FunctionType) interface{} {
	v.logVisit(node)
	if node.Name != "" {
		v.ExtractedValues["FunctionTypeName"] = node.Name
	}
	return nil
}
func (v *TestingVisitor) VisitInteger(node *Integer) interface{} {
	v.logVisit(node)
	v.ExtractedValues["IntegerValue"] = node.Value
	return nil
}
func (v *TestingVisitor) VisitString(node *String) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitDecoratedStatement(node *DecoratedStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitDocComment(node *DocComment) interface{} {
	v.logVisit(node)
	fullText := ""
	for _, fragment := range node.Fragments {
		if fragment.Text != "" {
			fullText += fragment.Text
		}
	}
	if fullText != "" {
		v.ExtractedValues["DocCommentText"] = fullText
	}
	return nil
}
func (v *TestingVisitor) VisitAnnotation(node *Annotation) interface{} {
	v.logVisit(node)
	v.ExtractedValues["AnnotationName"] = node.Name
	return nil
}
func (v *TestingVisitor) VisitIfBranch(node *IfBranch) interface{}     { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitCaseBranch(node *CaseBranch) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitStringFragment(node *StringFragment) interface{} {
	v.logVisit(node)
	if node.Text != "" {
		v.ExtractedValues["StringFragmentText"] = node.Text
	}
	return nil
}
func (v *TestingVisitor) VisitFunctionParameter(node *FunctionParameter) interface{} {
	v.logVisit(node)
	v.ExtractedValues["ParameterName"] = node.Name
	return nil
}
func (v *TestingVisitor) VisitOperand(node *Operand) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitPostfix(node *Postfix) interface{} {
	v.logVisit(node)
	if node.Operator != "" {
		v.ExtractedValues["PostfixOperator"] = node.Operator
	}
	return nil
}
