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
		Usages   []*Definition // The definitions that use this definition
	}
	// A context contains the root scope of an execution environment.
	Context struct {
		Root *Scope // The root scope
	}
)
