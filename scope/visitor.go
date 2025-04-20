package scope

import (
	"fmt"
	"reflect"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/cjhowedev/patina"
	"github.com/cjhowedev/patina/astutil"
)

// ScopeVisitor builds a tree of scopes and definitions from the AST.
type ScopeVisitor struct {
	astutil.NoOpVisitor // Embed NoOpVisitor for default behavior
	RootScope           *Scope
	currentScope        *Scope
	scopeStack          []*Scope
	Errors              []error
}

var _ astutil.Visitor = (*ScopeVisitor)(nil) // Compile-time check

// NewScopeVisitor creates a new visitor ready to build the scope tree.
func NewScopeVisitor() *ScopeVisitor {
	return &ScopeVisitor{
		scopeStack: []*Scope{},
		Errors:     []error{},
	}
}

// Helper to add errors during visitation.
func (v *ScopeVisitor) addError(node any, format string, args ...any) {
	pos := getPos(node) // Helper to extract position
	err := fmt.Errorf("scope error at %s: %w", pos, fmt.Errorf(format, args...))
	v.Errors = append(v.Errors, err)
}

// Visit implements the astutil.Visitor interface.
func (v *ScopeVisitor) Visit(node, parent any) (astutil.Visitor, astutil.ExitFunc, error) {
	var newScope *Scope
	pos := getPos(node)

	// Enter node: Handle scope creation and definitions
	switch n := node.(type) {
	case *patina.Root:
		// Root creates the global scope implicitly
		if v.currentScope == nil {
			globalScope := NewScope(n, nil, pos)
			v.RootScope = globalScope
			v.currentScope = globalScope
			v.scopeStack = append(v.scopeStack, globalScope)
			newScope = globalScope // Mark that this node created a scope
		}

	case *patina.Module:
		// Module definition creates a scope and is itself a definition in the parent scope
		parentScope := v.currentScope // Should be global scope
		if parentScope == nil {
			v.addError(n, "module '%s' declared outside global scope", n.Name)
			return nil, nil, fmt.Errorf("internal error: no current scope for module")
		}
		def := &Definition{Name: n.Name, Type: ModuleDefinition, Node: n, Position: n.Pos}
		if err := parentScope.Define(def); err != nil {
			v.addError(n, "%v", err)
		}
		// Module body creates its own scope
		// Pass the Module node itself as the node associated with the scope
		moduleScope := NewScope(n, parentScope, n.Body.Pos) // Use Body's position
		newScope = moduleScope                              // This node manages the scope for its children

	case *patina.FunctionStatement:
		// Function definition creates a scope and is a definition in the parent scope
		if v.currentScope == nil {
			v.addError(n, "function '%s' declared outside any scope", n.Name)
			return nil, nil, fmt.Errorf("internal error: no current scope for function")
		}
		def := &Definition{Name: n.Name, Type: FuncDefinition, Node: n, Position: n.Pos}
		if err := v.currentScope.Define(def); err != nil {
			v.addError(n, "%v", err)
		}
		// Function body/parameters create a new scope
		// Pass the FunctionStatement node as the node associated with the scope
		funcScope := NewScope(n, v.currentScope, n.Pos)
		newScope = funcScope // This node manages the scope for its children

		// Define parameters within the function's scope
		for _, param := range n.Parameters {
			paramDef := &Definition{Name: param.Name, Type: ParamDefinition, Node: param, Position: param.Pos}
			if err := funcScope.Define(paramDef); err != nil {
				// Error within the new function scope
				v.addError(param, "%v", err)
			}
		}

	case *patina.Block:
		// Block creates a new scope (unless it's a Module or Function body handled above)
		// Need to check parent to avoid double-scoping Module/Function bodies
		isModuleBody := false
		isFuncBody := false
		if pMod, ok := parent.(*patina.Module); ok {
			// Check if the Block's memory address matches the Module's Body address
			if &pMod.Body == n {
				isModuleBody = true
			}
		}
		if pFun, ok := parent.(*patina.FunctionStatement); ok {
			if &pFun.Block == n {
				isFuncBody = true
			}
		}

		if !isModuleBody && !isFuncBody && v.currentScope != nil {
			blockScope := NewScope(n, v.currentScope, n.Pos)
			// Optionally define the block's label if it has one and if that's desired semantics
			// if n.Label != "" { ... }
			newScope = blockScope
		}

	case *patina.LetStatement:
		// Variable definition
		if v.currentScope == nil {
			v.addError(n, "let statement for '%s' declared outside any scope", n.Name)
			return nil, nil, fmt.Errorf("internal error: no current scope for let statement")
		}
		def := &Definition{Name: n.Name, Type: VarDefinition, Node: n, Position: n.Pos}
		if err := v.currentScope.Define(def); err != nil {
			v.addError(n, "%v", err)
		}
		// Add cases for other definition types if necessary (e.g., type declarations)
	}

	// If this node created a new scope, push it onto the stack for children
	if newScope != nil {
		v.scopeStack = append(v.scopeStack, newScope)
		v.currentScope = newScope
	}

	// Define the exit function
	exit := func(childrenResults []any) (any, error) {
		// If this node created a scope, pop it from the stack when exiting
		if newScope != nil {
			if len(v.scopeStack) > 0 {
				v.scopeStack = v.scopeStack[:len(v.scopeStack)-1]
				if len(v.scopeStack) > 0 {
					v.currentScope = v.scopeStack[len(v.scopeStack)-1]
				} else {
					v.currentScope = nil // Should not happen if Root was handled correctly
				}
			}
		}
		// Scope visitor doesn't return hierarchical results, just builds the tree.
		return nil, nil
	}

	// Continue traversal with the same visitor
	return v, exit, nil
}

// Helper to get position from known AST node types
func getPos(node any) lexer.Position {
	// Use reflection to get the Pos field
	// This is boilerplate and could be improved if a Node interface exists
	v := reflect.ValueOf(node)
	if v.Kind() == reflect.Ptr {
		// Ensure the pointer is not nil before dereferencing
		if v.IsNil() {
			return lexer.Position{}
		}
		v = v.Elem()
	}
	// Ensure the value is a struct after potentially dereferencing
	if v.Kind() != reflect.Struct {
		return lexer.Position{}
	}
	f := v.FieldByName("Pos")
	if f.IsValid() && f.Type().AssignableTo(reflect.TypeOf(lexer.Position{})) {
		return f.Interface().(lexer.Position)
	}

	return lexer.Position{} // Return zero position if not found
}
