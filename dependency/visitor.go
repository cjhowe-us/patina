package dependency

import (
	"fmt"
	"reflect"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/cjhowedev/patina"
	"github.com/cjhowedev/patina/astutil"
	"github.com/cjhowedev/patina/scope"
)

// DependencyVisitor identifies dependencies between definitions using a pre-built scope tree.
type DependencyVisitor struct {
	astutil.NoOpVisitor                   // Embed NoOpVisitor
	RootScope           *scope.Scope      // The pre-built scope tree
	CurrentScope        *scope.Scope      // Tracks current scope during traversal
	DefinitionContext   *scope.Definition // The definition being currently analyzed (e.g., the LHS of let x = ...)
	Graph               DependencyGraph
	Errors              []error
	ScopeMap            map[uintptr]*scope.Scope // Map from AST node pointer address to the scope it defines/belongs to
}

var _ astutil.Visitor = (*DependencyVisitor)(nil) // Compile-time check

// NewDependencyVisitor creates a visitor to find dependencies.
// It requires the root scope obtained from a prior ScopeVisitor run and a map
// associating AST nodes with their corresponding scopes.
// ScopeMap key type changed from any to uintptr
func NewDependencyVisitor(rootScope *scope.Scope, scopeMap map[uintptr]*scope.Scope) *DependencyVisitor {
	if rootScope == nil {
		panic("DependencyVisitor requires a non-nil root scope")
	}
	if scopeMap == nil {
		panic("DependencyVisitor requires a non-nil scope map")
	}
	return &DependencyVisitor{
		RootScope:    rootScope,
		CurrentScope: rootScope, // Start at the root
		Graph:        make(DependencyGraph),
		Errors:       []error{},
		ScopeMap:     scopeMap,
	}
}

// BuildScopeMap is a helper function to create the node-to-scope map
// needed by the DependencyVisitor. It should be run after ScopeVisitor.
// Return type changed to map[uintptr]*scope.Scope
func BuildScopeMap(rootScope *scope.Scope) map[uintptr]*scope.Scope {
	// Key type changed to uintptr
	scopeMap := make(map[uintptr]*scope.Scope)
	var build func(s *scope.Scope)
	build = func(s *scope.Scope) {
		if s == nil {
			return
		}
		if s.Node != nil {
			// Use pointer address as the key if s.Node is a pointer
			nodeValue := reflect.ValueOf(s.Node)
			if nodeValue.Kind() == reflect.Ptr && !nodeValue.IsNil() {
				scopeMap[nodeValue.Pointer()] = s
			} else {
				// Handle non-pointer nodes if they can define scopes (log warning for now)
				// If non-pointer nodes *cannot* define scopes, this else can be removed.
				fmt.Printf("Warning: Scope node at %s is not a pointer, cannot reliably map. Type: %T\n", s.Position, s.Node)
			}
		}
		for _, child := range s.Children {
			build(child)
		}
	}
	build(rootScope)
	return scopeMap
}

// Helper to add errors during visitation.
func (v *DependencyVisitor) addError(node any, format string, args ...any) {
	pos := getPos(node) // Reusing the helper
	err := fmt.Errorf("dependency error at %s: %w", pos, fmt.Errorf(format, args...))
	v.Errors = append(v.Errors, err)
}

// Visit implements the astutil.Visitor interface.
func (v *DependencyVisitor) Visit(node, parent any) (astutil.Visitor, astutil.ExitFunc, error) {
	originalDefinitionContext := v.DefinitionContext // Store context before potentially changing it

	// Update CurrentScope based on the pre-built map using pointer address
	var nodePtr uintptr
	nodeValue := reflect.ValueOf(node)
	if nodeValue.Kind() == reflect.Ptr && !nodeValue.IsNil() {
		nodePtr = nodeValue.Pointer()
	}

	if nodePtr != 0 {
		if s, ok := v.ScopeMap[nodePtr]; ok {
			v.CurrentScope = s
		}
		// Removed else if CurrentScope == nil check, rely on map or parent scope
	} else if v.CurrentScope == nil {
		// Fallback only if node is not a mappable pointer AND we have no current scope
		// This might indicate an issue earlier in traversal or map building
		v.CurrentScope = v.RootScope
	}

	// If not found in map, we assume we are still within the scope of the parent node found in the map.
	// This relies on Walk visiting children before siblings.

	// Enter node: Set context or identify dependencies
	switch n := node.(type) {
	case *patina.LetStatement:
		// Find the definition for the variable being declared
		def, found := v.CurrentScope.Lookup(n.Name)
		if !found {
			// This indicates an issue with the scope tree or traversal order
			v.addError(n, "definition for let '%s' not found in current scope (%s)", n.Name, v.CurrentScope.Position)
		} else {
			v.DefinitionContext = def // Set context for visiting the value expression
		}

	case *patina.FunctionStatement:
		// Find the definition for the function itself (likely in the parent's scope)
		var def *scope.Definition
		var found bool
		if v.CurrentScope != nil {
			def, found = v.CurrentScope.Lookup(n.Name)
		} else if v.RootScope != nil {
			// Fallback for functions potentially defined directly under root? (Shouldn't happen with modules)
			def, found = v.RootScope.Lookup(n.Name)
		}

		if !found {
			v.addError(n, "definition for function '%s' not found in scope", n.Name)
		} else {
			v.DefinitionContext = def // Set context for visiting params and body
		}
		// Dependencies within parameter types and return types are handled when visiting those Type/Atom nodes.

	case *patina.Binding:
		// This is an identifier usage (e.g., a variable reference)
		if v.DefinitionContext != nil && v.CurrentScope != nil {
			// Look up the identifier in the current scope
			usedDef, found := v.CurrentScope.Lookup(n.Ident)
			if !found {
				v.addError(n, "identifier '%s' not found", n.Ident)
			} else {
				// Add a dependency from the context to the used definition
				v.Graph.AddDependency(v.DefinitionContext, usedDef)
			}
		}

	// Add cases for identifiers used in types (e.g., PointerType.Target) if needed
	// Consider type aliases or named types if they exist in the language.
	case *patina.PointerType:
		v.handleAtomDependency(n.Target)
	case *patina.ArrayType:
		v.handleAtomDependency(n.Element)
	case *patina.FunctionType:
		for _, param := range n.Parameters {
			v.handleAtomDependency(param.Type)
		}
		if n.Return != nil {
			v.handleAtomDependency(n.Return)
		}
	// Add more cases here for Atoms used within other expressions/statements if necessary
	// e.g., FunctionParameter.Type
	case *patina.FunctionParameter:
		v.handleAtomDependency(n.Type)

	}

	// Define the exit function
	exit := func(childrenResults []any) (any, error) {
		// Restore the definition context when exiting the node that set it.
		switch node.(type) {
		case *patina.LetStatement, *patina.FunctionStatement:
			v.DefinitionContext = originalDefinitionContext
		}

		// No need to restore CurrentScope here as the map lookup handles entering scopes.
		// The map ensures we are in the correct scope associated with the node.

		// Dependency visitor doesn't return hierarchical results.
		return nil, nil
	}

	// Continue traversal
	return v, exit, nil
}

// handleAtomDependency checks if an Atom is a Binding and adds a dependency if needed.
func (v *DependencyVisitor) handleAtomDependency(atom patina.Atom) {
	if binding, ok := atom.(*patina.Binding); ok {
		if v.DefinitionContext != nil && v.CurrentScope != nil {
			usedDef, found := v.CurrentScope.Lookup(binding.Ident)
			if !found {
				v.addError(binding, "type identifier '%s' not found", binding.Ident)
			} else {
				// Assuming type usage creates dependency on the type definition
				// Check if usedDef is actually a type definition if the language distinguishes
				v.Graph.AddDependency(v.DefinitionContext, usedDef)
			}
		}
	}
	// We might need to recurse if the Atom is a GroupedExpression containing a Binding, etc.
	// However, Walk should handle this automatically by visiting the children of the Atom.
}

// Helper to get position (reuse from scope visitor)
func getPos(node any) lexer.Position {
	v := reflect.ValueOf(node)
	if v.Kind() == reflect.Ptr {
		if v.IsNil() {
			return lexer.Position{}
		}
		v = v.Elem()
	}
	if v.Kind() != reflect.Struct {
		return lexer.Position{}
	}
	f := v.FieldByName("Pos")
	if f.IsValid() && f.Type().AssignableTo(reflect.TypeOf(lexer.Position{})) {
		return f.Interface().(lexer.Position)
	}
	return lexer.Position{}
}
