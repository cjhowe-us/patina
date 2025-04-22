package ast

import (
	"strings"
	"sync"
)

// Visitor is the interface implemented by AST visitors.
type Visitor[Input any, Output any] interface {
	Enter(node SyntaxNode, request *Input) ([]*Input, error)
	Exit(node SyntaxNode, childrenOutputs []*Output) (*Output, error)
}

// ErrorList represents a collection of errors that occurred during AST traversal.
type ErrorList []error

// Error returns a string representation of all errors in the list.
func (el ErrorList) Error() string {
	errStrings := make([]string, len(el))
	for i, err := range el {
		errStrings[i] = err.Error()
	}
	return "One or more errors occurred:\n  - " + strings.Join(errStrings, "\n  - ")
}

// Walk traverses an AST node recursively, calling the visitor's Enter and Exit methods for each node encountered. The
// traversal starts with the given visitor `v`. The `node` must be the root of the AST subtree to walk. The `input` is
// passed to the visitor's Enter method. The visitor's Exit method is called with the results of the traversal of the
// node's children.
func Walk[Input any, Output any](v Visitor[Input, Output], input *Input, node SyntaxNode) (*Output, error) {
	childInputs, err := v.Enter(node, input)
	if err != nil {
		return nil, err
	}

	children := node.Children()
	childrenResults := make([]*Output, len(children))
	var waitGroup sync.WaitGroup
	waitGroup.Add(len(children))
	errChan := make(chan error, len(children))
	defer close(errChan)
	for i, child := range children {
		go func(i int, child SyntaxNode) {
			defer waitGroup.Done()

			var input *Input
			if i < len(childInputs) {
				input = childInputs[i]
			} else {
				input = nil
			}
			childResult, err := Walk(v, input, child)
			if err != nil {
				errChan <- err
			}
			childrenResults[i] = childResult
		}(i, child)
	}

	waitGroup.Wait()

	errs := ErrorList{}
errloop:
	for {
		select {
		case val := <-errChan:
			errs = append(errs, val)
		default:
			break errloop
		}
	}
	if len(errs) > 0 {
		return nil, errs
	}

	return v.Exit(node, childrenResults)
}
