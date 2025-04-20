package astutil

// ExitFunc is called by Walk after visiting all children of a node.
// It receives the results returned by the ExitFuncs of its children.
type ExitFunc func(childrenResults []any) (any, error)

// Visitor is the interface implemented by AST visitors.
type Visitor interface {
	// Visit is called for each node visited by Walk.
	//
	// If the returned visitor `w` is not nil, Walk visits the children of the node
	// with `w` as the visitor. If `w` is nil, Walk skips the children.
	//
	// The returned `exit` function is called after visiting the children. If `exit`
	// is nil, a default exit function is used which returns nil.
	//
	// If an error is returned, Walk stops immediately.
	Visit(node, parent any) (w Visitor, exit ExitFunc, err error)
}

// VisitFunc is an adapter to allow the use of ordinary functions as Visitors.
// If f is a function with the appropriate signature, VisitFunc(f) is a
// Visitor that calls f.
type VisitFunc func(node, parent any) (Visitor, ExitFunc, error)

func (f VisitFunc) Visit(node, parent any) (Visitor, ExitFunc, error) {
	return f(node, parent)
}
