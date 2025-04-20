package astutil

import (
	"encoding/json"
	"fmt"
	"reflect"
)

// SnapshotNode represents a node visited during traversal, including the
// results returned by its children's exit functions.
// It includes basic info for easier marshaling.
type SnapshotNode struct {
	NodeType string `json:"type"`
	NodeStr  string `json:"nodeStr,omitempty"`  // String representation of the node
	Position string `json:"position,omitempty"` // Position in source
	// Node    any    `json:"-"`                   // Raw node excluded from JSON
	// Parent  any    `json:"-"`                   // Parent node excluded from JSON
	Results []any `json:"results,omitempty"` // Results from children
}

// SnapshotVisitor builds a tree mirroring the AST structure, where each node
// contains the results returned by its children's visitors.
// It attempts to create a JSON-marshalable representation.
type SnapshotVisitor struct{}

var _ Visitor = SnapshotVisitor{} // Compile-time check

func (s SnapshotVisitor) Visit(node, parent any) (Visitor, ExitFunc, error) {
	// Define the exit function for this node.
	exit := func(childrenResults []any) (any, error) {
		// Create the snapshot node for the current AST node.
		nodeType := reflect.TypeOf(node).String()
		posStr := ""
		var nodeStr string

		// Attempt to marshal the node itself to JSON for a stable string representation
		// This might fail if the node contains unmarshalable types (like functions) or cycles
		// For AST nodes, often custom marshalers or simpler representations are needed.
		if node != nil {
			nodeBytes, err := json.Marshal(node)
			if err != nil {
				// Fallback to less stable representation if JSON marshaling fails
				nodeStr = fmt.Sprintf("%T [JSON Marshal Error: %v]", node, err)
			} else {
				nodeStr = string(nodeBytes)
			}
		} else {
			nodeStr = "<nil>"
		}

		// Try to get position using reflection
		val := reflect.ValueOf(node)
		if val.Kind() == reflect.Ptr {
			val = val.Elem()
		}
		if val.Kind() == reflect.Struct {
			f := val.FieldByName("Pos")
			if f.IsValid() {
				pos, ok := f.Interface().(interface {
					String() string
				})
				if ok {
					posStr = pos.String()
				}
			}
			// Try to get a better string representation if available
			method := reflect.ValueOf(node).MethodByName("String")
			if method.IsValid() && method.Type().NumIn() == 0 && method.Type().NumOut() == 1 && method.Type().Out(0).Kind() == reflect.String {
				results := method.Call(nil)
				nodeStr = results[0].String()
			}
		}

		snapshot := SnapshotNode{
			NodeType: nodeType,
			NodeStr:  nodeStr,
			Position: posStr,
			Results:  childrenResults,
		}
		return snapshot, nil
	}

	// Continue traversal with the same visitor, and use the defined exit function.
	return s, exit, nil
}

// MarshalSnapshot marshals the result of a SnapshotVisitor run (which is typically a SnapshotNode)
// into a JSON byte slice with indentation.
func MarshalSnapshot(snapshotResult any) ([]byte, error) {
	return json.MarshalIndent(snapshotResult, "", "  ")
}
