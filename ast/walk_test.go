package ast

import (
	"encoding/json"
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
)

// MockNode is a simple implementation of SyntaxNode for testing purposes.
type MockNode struct {
	ID           string
	Pos          lexer.Position
	NodeChildren []SyntaxNode
}

func (n *MockNode) Position() lexer.Position {
	return n.Pos
}

func (n *MockNode) Children() []SyntaxNode {
	return n.NodeChildren
}

// TestVisitor tracks node visits for testing the Walk function.
type TestVisitor struct {
	VisitLog []string
}

// Enter implements the Visitor interface for TestVisitor.
func (v *TestVisitor) Enter(node SyntaxNode, request *string) ([]*string, error) {
	mockNode := node.(*MockNode)
	v.VisitLog = append(v.VisitLog, "Enter: "+mockNode.ID)

	inputs := make([]*string, len(mockNode.NodeChildren))
	for i := range inputs {
		childInput := "child-" + mockNode.ID + "-" + string(rune('0'+i))
		inputs[i] = &childInput
	}

	return inputs, nil
}

// Exit implements the Visitor interface for TestVisitor.
func (v *TestVisitor) Exit(node SyntaxNode, childrenOutputs []*string) (*string, error) {
	mockNode := node.(*MockNode)
	v.VisitLog = append(v.VisitLog, "Exit: "+mockNode.ID)

	result := mockNode.ID
	for _, output := range childrenOutputs {
		if output != nil {
			result += "-" + *output
		}
	}

	return &result, nil
}

// WalkInput represents input for a walk test.
type WalkInput struct {
	Tree  json.RawMessage `json:"tree"`
	Input string          `json:"input"`
}

// WalkTestOutput holds the output from a walk test.
type WalkTestOutput struct {
	Result   string   `json:"result"`
	VisitLog []string `json:"visitLog"`
}

// buildTree constructs a MockNode tree from its JSON representation.
func buildTree(data json.RawMessage) (*MockNode, error) {
	var nodeMap map[string]interface{}
	if err := json.Unmarshal(data, &nodeMap); err != nil {
		return nil, err
	}

	return buildNodeFromMap(nodeMap), nil
}

// buildNodeFromMap creates a MockNode from a parsed JSON map.
func buildNodeFromMap(nodeMap map[string]interface{}) *MockNode {
	node := &MockNode{
		ID: nodeMap["id"].(string),
		Pos: lexer.Position{
			Line:   int(nodeMap["line"].(float64)),
			Column: int(nodeMap["column"].(float64)),
		},
	}

	if childrenArray, ok := nodeMap["children"].([]interface{}); ok {
		node.NodeChildren = make([]SyntaxNode, len(childrenArray))

		for i, childMap := range childrenArray {
			node.NodeChildren[i] = buildNodeFromMap(childMap.(map[string]interface{}))
		}
	}

	return node
}

func TestWalk(t *testing.T) {
	RunTests(
		t,
		"walk.json",
		func(input WalkInput, name string) (WalkTestOutput, error) {
			tree, err := buildTree(input.Tree)
			if err != nil {
				return WalkTestOutput{}, err
			}

			visitor := &TestVisitor{
				VisitLog: []string{},
			}

			request := input.Input
			result, err := Walk(visitor, &request, tree)
			if err != nil {
				return WalkTestOutput{}, err
			}

			return WalkTestOutput{
				Result:   *result,
				VisitLog: visitor.VisitLog,
			}, nil
		},
		false,
	)
}
