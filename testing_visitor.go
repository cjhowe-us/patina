package patina

import "reflect"

type TestingVisitorResult struct {
	VisitLog  []string       `json:"visitLog"`
	NodeCount map[string]int `json:"nodeCount"`
}

type TestingVisitor struct {
	NoopVisitor

	VisitLog  []string
	NodeCount map[string]int
}

func NewTestingVisitor() *TestingVisitor {
	return &TestingVisitor{
		VisitLog:  make([]string, 0),
		NodeCount: make(map[string]int),
	}
}

func (v *TestingVisitor) logVisit(node interface{}) {
	nodeType := reflect.TypeOf(node).Elem().Name()
	v.VisitLog = append(v.VisitLog, nodeType)
	v.NodeCount[nodeType]++
}

func (v *TestingVisitor) Result() *TestingVisitorResult {
	return &TestingVisitorResult{
		VisitLog:  v.VisitLog,
		NodeCount: v.NodeCount,
	}
}

func (v *TestingVisitor) VisitRoot(node *Root) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitModule(node *Module) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitFunctionStatement(node *FunctionStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitReturnStatement(node *ReturnStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitLetStatement(node *LetStatement) interface{} {
	v.logVisit(node)
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
	return nil
}
func (v *TestingVisitor) VisitOperation(node *Operation) interface{} {
	v.logVisit(node)
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
	return nil
}
func (v *TestingVisitor) VisitPointerType(node *PointerType) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitArrayType(node *ArrayType) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitIntegerType(node *IntegerType) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitFunctionType(node *FunctionType) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitInteger(node *Integer) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitString(node *String) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitDecoratedStatement(node *DecoratedStatement) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitDocComment(node *DocComment) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitAnnotation(node *Annotation) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitIfBranch(node *IfBranch) interface{}     { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitCaseBranch(node *CaseBranch) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitStringFragment(node *StringFragment) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitFunctionParameter(node *FunctionParameter) interface{} {
	v.logVisit(node)
	return nil
}
func (v *TestingVisitor) VisitOperand(node *Operand) interface{} { v.logVisit(node); return nil }
func (v *TestingVisitor) VisitPostfix(node *Postfix) interface{} {
	v.logVisit(node)
	return nil
}
