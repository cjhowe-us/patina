package patina

type NoopVisitor struct{}

func (v *NoopVisitor) VisitRoot(node *Root) interface{}                               { return nil }
func (v *NoopVisitor) VisitModule(node *Module) interface{}                           { return nil }
func (v *NoopVisitor) VisitFunctionStatement(node *FunctionStatement) interface{}     { return nil }
func (v *NoopVisitor) VisitReturnStatement(node *ReturnStatement) interface{}         { return nil }
func (v *NoopVisitor) VisitLetStatement(node *LetStatement) interface{}               { return nil }
func (v *NoopVisitor) VisitBreakStatement(node *BreakStatement) interface{}           { return nil }
func (v *NoopVisitor) VisitExpressionStatement(node *ExpressionStatement) interface{} { return nil }
func (v *NoopVisitor) VisitIfExpression(node *IfExpression) interface{}               { return nil }
func (v *NoopVisitor) VisitCaseExpression(node *CaseExpression) interface{}           { return nil }
func (v *NoopVisitor) VisitLambda(node *Lambda) interface{}                           { return nil }
func (v *NoopVisitor) VisitStaticExpression(node *StaticExpression) interface{}       { return nil }
func (v *NoopVisitor) VisitTypeExpression(node *TypeExpression) interface{}           { return nil }
func (v *NoopVisitor) VisitBlock(node *Block) interface{}                             { return nil }
func (v *NoopVisitor) VisitOperation(node *Operation) interface{}                     { return nil }
func (v *NoopVisitor) VisitGroupedExpression(node *GroupedExpression) interface{}     { return nil }
func (v *NoopVisitor) VisitAtomicType(node *AtomicType) interface{}                   { return nil }
func (v *NoopVisitor) VisitAtomicValue(node *AtomicValue) interface{}                 { return nil }
func (v *NoopVisitor) VisitBinding(node *Binding) interface{}                         { return nil }
func (v *NoopVisitor) VisitPointerType(node *PointerType) interface{}                 { return nil }
func (v *NoopVisitor) VisitArrayType(node *ArrayType) interface{}                     { return nil }
func (v *NoopVisitor) VisitIntegerType(node *IntegerType) interface{}                 { return nil }
func (v *NoopVisitor) VisitFunctionType(node *FunctionType) interface{}               { return nil }
func (v *NoopVisitor) VisitInteger(node *Integer) interface{}                         { return nil }
func (v *NoopVisitor) VisitString(node *String) interface{}                           { return nil }
func (v *NoopVisitor) VisitDecoratedStatement(node *DecoratedStatement) interface{}   { return nil }
func (v *NoopVisitor) VisitDocComment(node *DocComment) interface{}                   { return nil }
func (v *NoopVisitor) VisitAnnotation(node *Annotation) interface{}                   { return nil }
func (v *NoopVisitor) VisitIfBranch(node *IfBranch) interface{}                       { return nil }
func (v *NoopVisitor) VisitCaseBranch(node *CaseBranch) interface{}                   { return nil }
func (v *NoopVisitor) VisitStringFragment(node *StringFragment) interface{}           { return nil }
func (v *NoopVisitor) VisitFunctionParameter(node *FunctionParameter) interface{}     { return nil }
func (v *NoopVisitor) VisitOperand(node *Operand) interface{}                         { return nil }
func (v *NoopVisitor) VisitPostfix(node *Postfix) interface{}                         { return nil }
