// This file implements AST traversal to discover scopes and definitions
package patina

// Reference represents a variable reference with its location and scope.
type Reference struct {
	Name  string
	Scope *Scope
	Def   *Definition // The definition that contains this reference
}

// DiscoverScopes traverses the AST and builds a scope hierarchy with definitions.
// It returns the root Context containing the discovered scopes and definitions.
func DiscoverScopes(root *Root) *Context {
	ctx := &Context{
		Root: NewScope("", nil, false),
	}

	// Track all references to build usage relationships
	var references []Reference

	// Process all top-level statements in the AST
	for _, decoratedStatement := range root.Statements {
		processStatement(decoratedStatement.Statement, ctx.Root, nil, &references)
	}

	// Build usage relationships between definitions
	buildUsageRelationships(ctx.Root, references)

	// Calculate compilation stages for all definitions
	CalculateStages(ctx)

	return ctx
}

// NewScope creates a new scope with the given name, parent, and static flag.
func NewScope(name string, parent *Scope, isStatic bool) *Scope {
	return &Scope{
		Name:     name,
		IsStatic: isStatic,
		defs:     make(map[string]*Definition),
		children: make(map[string]*Scope),
		parent:   parent,
	}
}

// processStatement handles different types of statements and updates the scope hierarchy.
func processStatement(stmt Statement, currentScope *Scope, currentDef *Definition, references *[]Reference) {
	switch s := stmt.(type) {
	case Module:
		// Create a new scope for the module
		moduleScope := NewScope(s.Name, currentScope, false)
		currentScope.children[s.Name] = moduleScope

		// Process statements in the module body
		for _, decoratedStatement := range s.Body.Statements {
			processStatement(decoratedStatement.Statement, moduleScope, nil, references)
		}

	case FunctionStatement:
		// Create a definition for the function
		funcDef := &Definition{
			Name:     s.Name,
			IsStatic: s.IsStatic,
			Parent:   currentScope,
			Usages:   nil,
		}
		currentScope.defs[s.Name] = funcDef

		// Create a new scope for the function body
		functionScope := NewScope(s.Name, currentScope, s.IsStatic)
		currentScope.children[s.Name] = functionScope

		// Process parameters as definitions in the function scope
		for _, param := range s.Parameters {
			paramDef := &Definition{
				Name:     param.Name,
				IsStatic: false,
				Parent:   functionScope,
				Usages:   nil,
			}
			functionScope.defs[param.Name] = paramDef
		}

		// Process function body statements
		for _, decoratedStatement := range s.Block.Statements {
			processStatement(decoratedStatement.Statement, functionScope, funcDef, references)
		}

	case LetStatement:
		// Create a definition for the let statement
		letDef := &Definition{
			Name:     s.Name,
			IsStatic: s.IsStatic,
			Parent:   currentScope,
			Usages:   nil,
		}
		currentScope.defs[s.Name] = letDef

		// Process value expression to collect variable references
		if s.Value != nil {
			processExpression(s.Value, currentScope, letDef, references)
		}

	case ReturnStatement:
		// Process the return value expression if it exists
		if s.Value != nil {
			processExpression(s.Value, currentScope, currentDef, references)
		}

	case BreakStatement:
		// Process the break value expression if it exists
		if s.Value != nil {
			processExpression(s.Value, currentScope, currentDef, references)
		}

	case ExpressionStatement:
		// Process the expression
		processExpression(s.Expression, currentScope, currentDef, references)
	}
}

// processExpression handles different types of expressions and collects variable references.
func processExpression(expr Expression, currentScope *Scope, currentDef *Definition, references *[]Reference) {
	switch e := expr.(type) {
	case IfExpression:
		// Process condition and branches
		processExpression(e.If.Condition, currentScope, currentDef, references)
		processExpression(e.If.Then, currentScope, currentDef, references)

		for _, elseIf := range e.ElseIfs {
			processExpression(elseIf.Condition, currentScope, currentDef, references)
			processExpression(elseIf.Then, currentScope, currentDef, references)
		}

		if e.Else != nil {
			processExpression(e.Else, currentScope, currentDef, references)
		}

	case CaseExpression:
		for _, branch := range e.Branches {
			if branch.If != nil {
				processExpression(branch.If.Condition, currentScope, currentDef, references)
				processExpression(branch.If.Then, currentScope, currentDef, references)
			}
			if branch.Else != nil {
				processExpression(branch.Else, currentScope, currentDef, references)
			}
		}

	case Lambda:
		// Create a new scope for the lambda body
		lambdaScope := NewScope("lambda", currentScope, false)
		currentScope.children["lambda"] = lambdaScope

		// Process parameters as definitions
		for _, param := range e.Parameters {
			paramDef := &Definition{
				Name:     param.Name,
				IsStatic: false,
				Parent:   lambdaScope,
				Usages:   nil,
			}
			lambdaScope.defs[param.Name] = paramDef
		}

		// Process lambda body statements
		for _, decoratedStatement := range e.Body.Statements {
			processStatement(decoratedStatement.Statement, lambdaScope, currentDef, references)
		}

	case StaticExpression:
		processExpression(e.Static, currentScope, currentDef, references)

	case Operation:
		// Process operands
		for _, operand := range e.Operands {
			processOperand(operand, currentScope, currentDef, references)
		}

	case Block:
		// Create a new scope for the block
		blockScope := NewScope("block", currentScope, false)
		currentScope.children["block"] = blockScope

		// Process block statements
		for _, decoratedStatement := range e.Statements {
			processStatement(decoratedStatement.Statement, blockScope, currentDef, references)
		}
	}
}

// processOperand handles processing of operands and their postfix operations.
func processOperand(op Operand, currentScope *Scope, currentDef *Definition, references *[]Reference) {
	// Process the atom
	processAtom(op.Atom, currentScope, currentDef, references)

	// Process postfix operations
	for _, postfix := range op.Postfix {
		// Process function invocation arguments
		for _, arg := range postfix.Invocation {
			processExpression(arg, currentScope, currentDef, references)
		}

		// Process index expression
		if postfix.Index != nil {
			processExpression(postfix.Index, currentScope, currentDef, references)
		}
	}
}

// processAtom handles processing of atomic expressions.
func processAtom(atom Atom, currentScope *Scope, currentDef *Definition, references *[]Reference) {
	switch a := atom.(type) {
	case GroupedExpression:
		processExpression(a.Expression, currentScope, currentDef, references)

	case Binding:
		// Record the binding as a reference
		if currentDef != nil {
			// Only record references if we know what definition is making the reference
			recordBindingReference(a.Ident, currentScope, currentDef, references)
		}
	}
	// Other atom types like AtomicType and AtomicValue don't contain references
}

// recordBindingReference records a reference to a binding in the current scope or parent scopes.
func recordBindingReference(name string, currentScope *Scope, currentDef *Definition, references *[]Reference) {
	// Add the reference to our tracking list
	*references = append(*references, Reference{
		Name:  name,
		Scope: currentScope,
		Def:   currentDef,
	})
}

// buildUsageRelationships establishes usage relationships between definitions.
// This should be called after the entire scope hierarchy has been built.
func buildUsageRelationships(rootScope *Scope, references []Reference) {
	// Process all references and connect them to their definitions
	for _, ref := range references {
		// Find the definition being referenced
		usedDef := findDefinition(ref.Name, ref.Scope)
		if usedDef != nil && ref.Def != nil {
			// Add the reference to the definition's usage list
			// This creates a relationship: ref.Def uses usedDef
			usedDef.Usages = append(usedDef.Usages, ref.Def)
		}
	}
}

// findDefinition looks up a definition by name in the current scope or parent scopes.
func findDefinition(name string, currentScope *Scope) *Definition {
	// Check current scope
	if def, ok := currentScope.defs[name]; ok {
		return def
	}

	// Check parent scope
	if currentScope.parent != nil {
		return findDefinition(name, currentScope.parent)
	}

	// Definition not found
	return nil
}
