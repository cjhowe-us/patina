package astutil

import (
	"reflect"
)

// Walk traverses an AST node recursively, calling the visitor's Visit method
// for each node encountered. The traversal starts with the given visitor `v`.
// The `node` must be the root of the AST subtree to walk.
func Walk(v Visitor, node any) (any, error) {
	// Initial call doesn't have a parent
	return walk(v, node, nil)
}

// walk is the recursive helper function.
func walk(v Visitor, node, parent any) (any, error) {
	// Ensure node is not a nil interface/pointer before proceeding
	nodeValue := reflect.ValueOf(node)
	if !nodeValue.IsValid() || (nodeValue.Kind() == reflect.Ptr || nodeValue.Kind() == reflect.Interface) && nodeValue.IsNil() {
		return nil, nil
	}

	// Call the visitor's Visit method for the current node
	w, exit, err := v.Visit(node, parent)
	if err != nil {
		return nil, err // Propagate error immediately
	}

	// If the returned visitor is nil, skip visiting children
	if w == nil {
		// If an exit function was provided even when skipping children, call it.
		if exit != nil {
			return exit(nil) // Pass nil since no children were visited
		}
		return nil, nil // No visitor, no exit func, just return nil
	}

	// Prepare to collect results from children
	var childrenResults []any

	// Use reflection to iterate over children
	// Dereference pointer/interface to access fields/elements
	val := nodeValue
	if val.Kind() == reflect.Ptr || val.Kind() == reflect.Interface {
		// Check again for nil after dereferencing pointers/interfaces
		if val.IsNil() {
			if exit != nil {
				return exit(nil)
			}
			return nil, nil
		}
		val = val.Elem()
	}

	// Handle cases where the underlying value might still be invalid (e.g., nil map)
	if !val.IsValid() {
		if exit != nil {
			return exit(nil)
		}
		return nil, nil
	}

	switch val.Kind() {
	case reflect.Struct:
		for i := 0; i < val.NumField(); i++ {
			// Check if field is exported before accessing Interface()
			if !val.Type().Field(i).IsExported() {
				continue
			}
			fieldVal := val.Field(i)
			// Get the actual field value (could be struct, ptr, slice, interface, etc.)
			childNode := fieldVal.Interface()
			// Recursively walk this child
			childResult, err := walk(w, childNode, node) // Pass original node as parent
			if err != nil {
				return nil, err // Propagate child error
			}
			// Append non-nil results (visitors decide what's meaningful)
			if childResult != nil {
				childrenResults = append(childrenResults, childResult)
			}
		}
	case reflect.Slice:
		for i := 0; i < val.Len(); i++ {
			elemVal := val.Index(i)
			childNode := elemVal.Interface()
			childResult, err := walk(w, childNode, node) // Pass original node as parent
			if err != nil {
				return nil, err // Propagate child error
			}
			if childResult != nil {
				childrenResults = append(childrenResults, childResult)
			}
		}
		// Add Map handling if necessary, although less common for ASTs
	}

	// Call the exit function after visiting all children
	if exit != nil {
		return exit(childrenResults)
	}

	// Default exit: return nil if no exit function provided
	return nil, nil
}
