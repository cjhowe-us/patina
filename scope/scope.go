package scope

import (
	"fmt"

	"github.com/alecthomas/participle/v2/lexer"
	_ "github.com/cjhowedev/patina" // Import the AST package for type references if needed, aliased to _ if only for side-effects/type resolution
)

// DefinitionType indicates the kind of entity being defined.
type DefinitionType int

const (
	VarDefinition DefinitionType = iota
	FuncDefinition
	TypeDefinition // For named types if the language supports them
	ModuleDefinition
	ParamDefinition
	ScopeDefinition // For anonymous blocks/scopes
)

// Definition represents a named entity declared in the source code.
type Definition struct {
	Name     string
	Type     DefinitionType
	Node     any    // The AST node where the definition occurs (e.g., LetStatement, FunctionStatement)
	Scope    *Scope // The scope this definition belongs to
	Position lexer.Position
	// Add fields for type information, dependencies etc. as needed later
	Dependencies []*Definition `json:"-"` // Avoid cycles in default JSON
}

func (d *Definition) String() string {
	return fmt.Sprintf("%s (%s) at %s", d.Name, d.Type, d.Position)
}

func (dt DefinitionType) String() string {
	switch dt {
	case VarDefinition:
		return "Var"
	case FuncDefinition:
		return "Func"
	case TypeDefinition:
		return "Type"
	case ModuleDefinition:
		return "Module"
	case ParamDefinition:
		return "Param"
	case ScopeDefinition:
		return "Scope"
	default:
		return "Unknown"
	}
}

// Scope represents a lexical scope containing definitions.
type Scope struct {
	Node        any // The AST node that creates this scope (e.g., Module, FunctionStatement, Block)
	Parent      *Scope
	Children    []*Scope
	Definitions map[string]*Definition
	Position    lexer.Position
}

// NewScope creates a new scope associated with an AST node and a parent scope.
func NewScope(node any, parent *Scope, pos lexer.Position) *Scope {
	s := &Scope{
		Node:        node,
		Parent:      parent,
		Children:    []*Scope{},
		Definitions: make(map[string]*Definition),
		Position:    pos,
	}
	if parent != nil {
		parent.Children = append(parent.Children, s)
	}
	return s
}

// Define adds a definition to the scope. Returns an error if the name is already defined.
func (s *Scope) Define(def *Definition) error {
	if _, exists := s.Definitions[def.Name]; exists {
		// Consider allowing shadowing based on language rules, but error for now.
		return fmt.Errorf("'%s' already defined in scope at %s (previous definition at %s)",
			def.Name, def.Position, s.Definitions[def.Name].Position)
	}
	def.Scope = s
	s.Definitions[def.Name] = def
	return nil
}

// Lookup finds a definition by name, searching upwards through parent scopes.
func (s *Scope) Lookup(name string) (*Definition, bool) {
	current := s
	for current != nil {
		if def, found := current.Definitions[name]; found {
			return def, true
		}
		current = current.Parent
	}
	return nil, false
}
