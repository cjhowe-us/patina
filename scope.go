package patina

type (
	Stage uint64
	// A scope is a named, hierarchical container of definitions.
	Scope struct {
		Name     string // The name of the scope
		IsStatic bool   // Whether the scope is static

		defs     map[string]*Definition // Definitions in the scope
		children map[string]*Scope      // Child scopes
		parent   *Scope                 // Parent scope
	}
	// A definition is a named entity in a scope.
	Definition struct {
		Name     string        // The name of the definition
		IsStatic bool          // Whether the definition is static
		Stage    Stage         // Compilation stage of the definition
		Parent   *Scope        // Parent scope of the definition
		Usages   []*Definition // The definitions this definition depends on (dependencies)
		AST      interface{}   // The AST node (LetStatement or FunctionStatement) that defines this
	}
	// A context contains the root scope of an execution environment.
	Context struct {
		Root *Scope // The root scope
	}

	// DefinitionDiscoveryContext provides context for the definition discovery process
	DefinitionDiscoveryContext struct {
		CurrentScope *Scope
		CurrentDef   *Definition
	}

	// DefinitionDiscoverer is an interface for discovering definitions in AST nodes
	DefinitionDiscoverer interface {
		DiscoverDefinitions(ctx *DefinitionDiscoveryContext)
	}
)

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

// FindDefinition looks up a definition by name in the current scope or parent scopes.
func FindDefinition(name string, currentScope *Scope) *Definition {
	// Check current scope
	if def, ok := currentScope.defs[name]; ok {
		return def
	}

	// Check parent scope
	if currentScope.parent != nil {
		return FindDefinition(name, currentScope.parent)
	}

	// Definition not found
	return nil
}

// AddDefinition adds a definition to the scope
func (s *Scope) AddDefinition(def *Definition) {
	s.defs[def.Name] = def
}

// AddChildScope adds a child scope to the scope
func (s *Scope) AddChildScope(childScope *Scope) {
	s.children[childScope.Name] = childScope
}

// GetDefinitions returns all definitions in the scope
func (s *Scope) GetDefinitions() map[string]*Definition {
	return s.defs
}

// GetChildren returns all child scopes of the scope
func (s *Scope) GetChildren() map[string]*Scope {
	return s.children
}

// GetParent returns the parent scope
func (s *Scope) GetParent() *Scope {
	return s.parent
}
