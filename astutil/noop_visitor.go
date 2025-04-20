package astutil

// NoOpVisitor is a visitor that does nothing. It can be embedded in other
// visitors to provide default implementations.
type NoOpVisitor struct{}

var _ Visitor = NoOpVisitor{} // Compile-time check

func (v NoOpVisitor) Visit(node, parent any) (Visitor, ExitFunc, error) {
	// Continue traversal with the same visitor, no exit action needed.
	return v, nil, nil
}

// Helper function to get a default exit func if needed elsewhere.
func NoOpExitFunc(childrenResults []any) (any, error) {
	return nil, nil
}
