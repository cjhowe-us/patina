// This file implements AST traversal to discover scopes and definitions
package patina

// DiscoverScopes traverses the AST and builds a scope hierarchy with definitions.
// It returns the root Context containing the discovered scopes and definitions.
func DiscoverScopes(root *Root) *Context {
	ctx := &Context{
		Root: NewScope("", nil, false),
	}

	// Process all top-level statements in the AST
	discoveryContext := &DefinitionDiscoveryContext{
		CurrentScope: ctx.Root,
		CurrentDef:   nil,
	}

	for _, decoratedStatement := range root.Statements {
		decoratedStatement.Statement.DiscoverDefinitions(discoveryContext)
	}

	// Build usage relationships between definitions (DFS)
	buildUsageRelationships(ctx.Root)

	// Calculate compilation stages for all definitions
	err := CalculateStages(ctx)
	if err != nil {
		// In a real compiler, we'd properly handle this error
		// For now, we'll just return the context and let the caller handle the error
		return ctx
	}

	return ctx
}

// Implementation of DefinitionDiscoverer for Module
func (m Module) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Create a new scope for the module
	moduleScope := NewScope(m.Name, ctx.CurrentScope, false)
	ctx.CurrentScope.AddChildScope(moduleScope)

	// Process statements in the module body
	prevScope := ctx.CurrentScope
	ctx.CurrentScope = moduleScope

	for _, decoratedStatement := range m.Body.Statements {
		decoratedStatement.Statement.DiscoverDefinitions(ctx)
	}

	// Restore original scope
	ctx.CurrentScope = prevScope
}

// Implementation of DefinitionDiscoverer for FunctionStatement
func (f FunctionStatement) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Create a definition for the function
	funcDef := &Definition{
		Name:     f.Name,
		IsStatic: f.IsStatic,
		Parent:   ctx.CurrentScope,
		Usages:   nil,
		AST:      f, // Store the AST node
	}
	ctx.CurrentScope.AddDefinition(funcDef)

	// Create a new scope for the function body
	functionScope := NewScope(f.Name, ctx.CurrentScope, f.IsStatic)
	ctx.CurrentScope.AddChildScope(functionScope)

	// Save the current context values
	prevScope := ctx.CurrentScope
	prevDef := ctx.CurrentDef

	// Set the new context values
	ctx.CurrentScope = functionScope
	ctx.CurrentDef = funcDef

	// Process parameters as definitions in the function scope
	for _, param := range f.Parameters {
		paramDef := &Definition{
			Name:     param.Name,
			IsStatic: false,
			Parent:   functionScope,
			Usages:   nil,
		}
		functionScope.AddDefinition(paramDef)

		// Process parameter type
		param.Type.DiscoverDefinitions(ctx)
	}

	// Process return type if it exists
	if f.Return != nil {
		f.Return.DiscoverDefinitions(ctx)
	}

	// Process function body statements
	for _, decoratedStatement := range f.Block.Statements {
		decoratedStatement.Statement.DiscoverDefinitions(ctx)
	}

	// Restore original context
	ctx.CurrentScope = prevScope
	ctx.CurrentDef = prevDef
}

// Implementation of DefinitionDiscoverer for LetStatement
func (l LetStatement) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Create a definition for the let statement
	letDef := &Definition{
		Name:     l.Name,
		IsStatic: l.IsStatic,
		Parent:   ctx.CurrentScope,
		Usages:   nil,
		AST:      l, // Store the AST node
	}
	ctx.CurrentScope.AddDefinition(letDef)

	// Process type if it exists
	if l.Type != nil {
		l.Type.DiscoverDefinitions(ctx)
	}

	// Process value expression to collect variable references
	if l.Value != nil {
		// Save the current def
		prevDef := ctx.CurrentDef
		ctx.CurrentDef = letDef

		l.Value.DiscoverDefinitions(ctx)

		// Restore original def
		ctx.CurrentDef = prevDef
	}
}

// Implementation of DefinitionDiscoverer for ReturnStatement
func (r ReturnStatement) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Process the return value expression if it exists
	if r.Value != nil {
		r.Value.DiscoverDefinitions(ctx)
	}
}

// Implementation of DefinitionDiscoverer for BreakStatement
func (b BreakStatement) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Process the break value expression if it exists
	if b.Value != nil {
		b.Value.DiscoverDefinitions(ctx)
	}
}

// Implementation of DefinitionDiscoverer for ExpressionStatement
func (e ExpressionStatement) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Process the expression
	e.Expression.DiscoverDefinitions(ctx)
}

// Implementation of DefinitionDiscoverer for IfExpression
func (i IfExpression) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Process condition and branches
	i.If.Condition.DiscoverDefinitions(ctx)
	i.If.Then.DiscoverDefinitions(ctx)

	for _, elseIf := range i.ElseIfs {
		elseIf.Condition.DiscoverDefinitions(ctx)
		elseIf.Then.DiscoverDefinitions(ctx)
	}

	if i.Else != nil {
		i.Else.DiscoverDefinitions(ctx)
	}
}

// Implementation of DefinitionDiscoverer for CaseExpression
func (c CaseExpression) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	for _, branch := range c.Branches {
		if branch.If != nil {
			branch.If.Condition.DiscoverDefinitions(ctx)
			branch.If.Then.DiscoverDefinitions(ctx)
		}
		if branch.Else != nil {
			branch.Else.DiscoverDefinitions(ctx)
		}
	}
}

// Implementation of DefinitionDiscoverer for Lambda
func (l Lambda) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Create a new scope for the lambda body
	lambdaScope := NewScope("lambda", ctx.CurrentScope, false)
	ctx.CurrentScope.AddChildScope(lambdaScope)

	// Save the current scope
	prevScope := ctx.CurrentScope
	ctx.CurrentScope = lambdaScope

	// Process parameters as definitions
	for _, param := range l.Parameters {
		paramDef := &Definition{
			Name:     param.Name,
			IsStatic: false,
			Parent:   lambdaScope,
			Usages:   nil,
		}
		lambdaScope.AddDefinition(paramDef)

		// Process parameter type
		param.Type.DiscoverDefinitions(ctx)
	}

	// Process return type if it exists
	if l.Return != nil {
		l.Return.DiscoverDefinitions(ctx)
	}

	// Process lambda body statements
	for _, decoratedStatement := range l.Body.Statements {
		decoratedStatement.Statement.DiscoverDefinitions(ctx)
	}

	// Restore original scope
	ctx.CurrentScope = prevScope
}

// Implementation of DefinitionDiscoverer for StaticExpression
func (s StaticExpression) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	s.Static.DiscoverDefinitions(ctx)
}

// Implementation of DefinitionDiscoverer for TypeExpression
func (t TypeExpression) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// No definitions to discover in a type expression
}

// Implementation of DefinitionDiscoverer for Block
func (b Block) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Create a new scope for the block
	blockScope := NewScope("block", ctx.CurrentScope, false)
	ctx.CurrentScope.AddChildScope(blockScope)

	// Save the current scope
	prevScope := ctx.CurrentScope
	ctx.CurrentScope = blockScope

	// Process block statements
	for _, decoratedStatement := range b.Statements {
		decoratedStatement.Statement.DiscoverDefinitions(ctx)
	}

	// Restore original scope
	ctx.CurrentScope = prevScope
}

// Implementation of DefinitionDiscoverer for Operation
func (o Operation) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// First process all atoms in the operands
	for _, operand := range o.Operands {
		operand.Atom.DiscoverDefinitions(ctx)
	}

	// Then process all postfix operations
	for _, operand := range o.Operands {
		for _, postfix := range operand.Postfix {
			// Process function invocation arguments
			for _, arg := range postfix.Invocation {
				arg.DiscoverDefinitions(ctx)
			}

			// Process index expression
			if postfix.Index != nil {
				postfix.Index.DiscoverDefinitions(ctx)
			}
		}
	}
}

// Implementation of DefinitionDiscoverer for GroupedExpression
func (g GroupedExpression) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	g.Expression.DiscoverDefinitions(ctx)
}

// Implementation of DefinitionDiscoverer for AtomicType
func (a AtomicType) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// No definitions to discover in an atomic type
}

// Implementation of DefinitionDiscoverer for AtomicValue
func (a AtomicValue) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// No definitions to discover in an atomic value
}

// Binding represents a variable reference.
// When encountered during scope discovery, these create references between definitions.
func (b Binding) DiscoverDefinitions(ctx *DefinitionDiscoveryContext) {
	// Do nothing here. Usages are collected in buildUsageRelationships/traverseDefinition only.
}

// Add a new interface for collecting usages
// This mirrors the DefinitionDiscoverer pattern

type UsageCollector interface {
	CollectUsages(def *Definition, scope *Scope)
}

// Implement UsageCollector for all relevant AST node types

func (l LetStatement) CollectUsages(def *Definition, scope *Scope) {
	if l.Value != nil {
		collectUsagesFromExpr(def, l.Value, scope)
	}
}

func (f FunctionStatement) CollectUsages(def *Definition, scope *Scope) {
	funcScope := scope.children[def.Name]
	for _, stmt := range f.Block.Statements {
		collectUsagesFromStatement(def, stmt.Statement, funcScope)
	}
}

func (b Block) CollectUsages(def *Definition, scope *Scope) {
	blockScope := scope.children["block"]
	for _, d := range blockScope.defs {
		if d.AST != nil {
			if uc, ok := d.AST.(UsageCollector); ok {
				uc.CollectUsages(d, blockScope)
			}
		}
	}
}

func (l Lambda) CollectUsages(def *Definition, scope *Scope) {
	lambdaScope := scope.children["lambda"]
	for _, d := range lambdaScope.defs {
		if d.AST != nil {
			if uc, ok := d.AST.(UsageCollector); ok {
				uc.CollectUsages(d, lambdaScope)
			}
		}
	}
}

func (e ExpressionStatement) CollectUsages(def *Definition, scope *Scope) {
	if uc, ok := e.Expression.(UsageCollector); ok {
		uc.CollectUsages(def, scope)
	}
}

// For expressions, implement UsageCollector as needed
func (o Operation) CollectUsages(def *Definition, scope *Scope) {
	for _, operand := range o.Operands {
		if uc, ok := operand.Atom.(UsageCollector); ok {
			uc.CollectUsages(def, scope)
		}
		for _, postfix := range operand.Postfix {
			for _, arg := range postfix.Invocation {
				if uc, ok := arg.(UsageCollector); ok {
					uc.CollectUsages(def, scope)
				}
			}
			if postfix.Index != nil {
				if uc, ok := postfix.Index.(UsageCollector); ok {
					uc.CollectUsages(def, scope)
				}
			}
		}
	}
}

func (g GroupedExpression) CollectUsages(def *Definition, scope *Scope) {
	if uc, ok := g.Expression.(UsageCollector); ok {
		uc.CollectUsages(def, scope)
	}
}

func (b Binding) CollectUsages(def *Definition, scope *Scope) {
	usedDef := FindDefinition(b.Ident, scope)
	if usedDef != nil {
		// Record the usage, even if it's a self-reference
		// Self-references will be handled by the cycle detection logic
		already := false
		for _, u := range def.Usages {
			if u == usedDef {
				already = true
				break
			}
		}
		if !already {
			def.Usages = append(def.Usages, usedDef)
		}
	}
}

// For all other statement types, no-op
func (ReturnStatement) CollectUsages(def *Definition, scope *Scope) {}
func (BreakStatement) CollectUsages(def *Definition, scope *Scope)  {}

// buildUsageRelationships now recursively walks all definitions in all scopes
func buildUsageRelationships(rootScope *Scope) {
	var walkDefs func(scope *Scope)
	walkDefs = func(scope *Scope) {
		for _, def := range scope.defs {
			if def.AST == nil {
				continue
			}
			if uc, ok := def.AST.(UsageCollector); ok {
				uc.CollectUsages(def, scope)
			}
		}
		for _, child := range scope.children {
			walkDefs(child)
		}
	}
	walkDefs(rootScope)
}

func collectUsagesFromStatement(def *Definition, stmt Statement, scope *Scope) {
	if collector, ok := stmt.(StatementUsageCollector); ok {
		collector.CollectStatementUsages(def, scope)
		return
	}
}

func collectUsagesFromExpr(def *Definition, expr Expression, scope *Scope) {
	if collector, ok := expr.(ExpressionUsageCollector); ok {
		collector.CollectExpressionUsages(def, scope)
		return
	}
}

func collectUsagesFromAtom(def *Definition, atom Atom, scope *Scope) {
	if collector, ok := atom.(AtomUsageCollector); ok {
		collector.CollectAtomUsages(def, scope)
		return
	}
}

// StatementUsageCollector represents a statement that can collect usage information
type StatementUsageCollector interface {
	CollectStatementUsages(def *Definition, scope *Scope)
}

// Implementation of StatementUsageCollector for LetStatement
func (l LetStatement) CollectStatementUsages(def *Definition, scope *Scope) {
	if l.Value != nil {
		collectUsagesFromExpr(def, l.Value, scope)
	}
}

// Implementation of StatementUsageCollector for ReturnStatement
func (r ReturnStatement) CollectStatementUsages(def *Definition, scope *Scope) {
	if r.Value != nil {
		collectUsagesFromExpr(def, r.Value, scope)
	}
}

// Implementation of StatementUsageCollector for BreakStatement
func (b BreakStatement) CollectStatementUsages(def *Definition, scope *Scope) {
	if b.Value != nil {
		collectUsagesFromExpr(def, b.Value, scope)
	}
}

// Implementation of StatementUsageCollector for ExpressionStatement
func (e ExpressionStatement) CollectStatementUsages(def *Definition, scope *Scope) {
	collectUsagesFromExpr(def, e.Expression, scope)
}

// Implementation of StatementUsageCollector for FunctionStatement
func (f FunctionStatement) CollectStatementUsages(def *Definition, scope *Scope) {
	functionScope := scope.children[f.Name]
	for _, stmt := range f.Block.Statements {
		collectUsagesFromStatement(def, stmt.Statement, functionScope)
	}
}

// ExpressionUsageCollector represents an expression that can collect usage information
type ExpressionUsageCollector interface {
	CollectExpressionUsages(def *Definition, scope *Scope)
}

// AtomUsageCollector represents an atom that can collect usage information
type AtomUsageCollector interface {
	CollectAtomUsages(def *Definition, scope *Scope)
}

// Implementation of ExpressionUsageCollector for Operation
func (o Operation) CollectExpressionUsages(def *Definition, scope *Scope) {
	for _, operand := range o.Operands {
		collectUsagesFromAtom(def, operand.Atom, scope)
		for _, postfix := range operand.Postfix {
			for _, arg := range postfix.Invocation {
				collectUsagesFromExpr(def, arg, scope)
			}
			if postfix.Index != nil {
				collectUsagesFromExpr(def, postfix.Index, scope)
			}
		}
	}
}

// Implementation of ExpressionUsageCollector for IfExpression
func (i IfExpression) CollectExpressionUsages(def *Definition, scope *Scope) {
	collectUsagesFromExpr(def, i.If.Condition, scope)
	collectUsagesFromExpr(def, i.If.Then, scope)
	for _, elseIf := range i.ElseIfs {
		collectUsagesFromExpr(def, elseIf.Condition, scope)
		collectUsagesFromExpr(def, elseIf.Then, scope)
	}
	if i.Else != nil {
		collectUsagesFromExpr(def, i.Else, scope)
	}
}

// Implementation of ExpressionUsageCollector for CaseExpression
func (c CaseExpression) CollectExpressionUsages(def *Definition, scope *Scope) {
	for _, branch := range c.Branches {
		if branch.If != nil {
			collectUsagesFromExpr(def, branch.If.Condition, scope)
			collectUsagesFromExpr(def, branch.If.Then, scope)
		}
		if branch.Else != nil {
			collectUsagesFromExpr(def, branch.Else, scope)
		}
	}
}

// Implementation of ExpressionUsageCollector for Lambda
func (l Lambda) CollectExpressionUsages(def *Definition, scope *Scope) {
	lambdaScope := scope.children["lambda"]
	if lambdaScope != nil {
		// Process lambda body statements
		for _, stmt := range l.Body.Statements {
			collectUsagesFromStatement(def, stmt.Statement, lambdaScope)
		}

		// Process definitions in the lambda scope
		for _, lambdaDef := range lambdaScope.defs {
			// Add usage for each definition in the lambda
			already := false
			for _, u := range def.Usages {
				if u == lambdaDef {
					already = true
					break
				}
			}
			if !already && def != lambdaDef {
				def.Usages = append(def.Usages, lambdaDef)
			}
		}
	}
}

// Implementation of ExpressionUsageCollector for StaticExpression
func (s StaticExpression) CollectExpressionUsages(def *Definition, scope *Scope) {
	collectUsagesFromExpr(def, s.Static, scope)
}

// Implementation of ExpressionUsageCollector for Block
func (b Block) CollectExpressionUsages(def *Definition, scope *Scope) {
	blockScope := scope.children["block"]
	if blockScope != nil {
		// Process block statements
		for _, stmt := range b.Statements {
			collectUsagesFromStatement(def, stmt.Statement, blockScope)
		}

		// Process definitions in the block scope
		for _, blockDef := range blockScope.defs {
			// Add usage for each definition in the block
			already := false
			for _, u := range def.Usages {
				if u == blockDef {
					already = true
					break
				}
			}
			if !already && def != blockDef {
				def.Usages = append(def.Usages, blockDef)
			}
		}
	}
}

// Implementation of AtomUsageCollector for Binding
func (b Binding) CollectAtomUsages(def *Definition, scope *Scope) {
	usedDef := FindDefinition(b.Ident, scope)
	if usedDef != nil {
		// Include all usages, even self-references
		already := false
		for _, u := range def.Usages {
			if u == usedDef {
				already = true
				break
			}
		}
		if !already {
			def.Usages = append(def.Usages, usedDef)
		}
	}
}

// Implementation of AtomUsageCollector for GroupedExpression
func (g GroupedExpression) CollectAtomUsages(def *Definition, scope *Scope) {
	collectUsagesFromExpr(def, g.Expression, scope)
}
