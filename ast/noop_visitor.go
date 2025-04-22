package ast

// NoOpVisitor implements a visitor that performs no operations.
type NoOpVisitor[Input any, Output any] struct{}

func (v *NoOpVisitor[Input, Output]) Enter(node SyntaxNode, request *Input) ([]*Input, error) {
	return nil, nil
}

func (v *NoOpVisitor[Input, Output]) Exit(node SyntaxNode, childrenOutputs []*Output) (*Output, error) {
	return nil, nil
}

func NoOpExitFunc(childrenResults []any) (any, error) {
	return nil, nil
}
