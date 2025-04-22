package ast

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/gkampitakis/go-snaps/snaps"
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

// TestCase represents a test case for the Walk function.
type TestCase struct {
	Name  string          `json:"name"`
	Tree  json.RawMessage `json:"tree"`
	Input string          `json:"input"`
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

// loadTestCases loads test cases from the JSON file.
func loadTestCases() ([]TestCase, error) {
	fileData, err := os.ReadFile(filepath.Join("testdata", "walk_test_cases.json"))
	if err != nil {
		return nil, err
	}

	var testCases []TestCase
	if err := json.Unmarshal(fileData, &testCases); err != nil {
		return nil, err
	}

	return testCases, nil
}

func TestWalk(t *testing.T) {
	testCases, err := loadTestCases()
	if err != nil {
		t.Fatalf("Failed to load test cases: %v", err)
	}

	for _, tc := range testCases {
		t.Run(tc.Name, func(t *testing.T) {
			tree, err := buildTree(tc.Tree)
			if err != nil {
				t.Fatalf("Failed to build tree: %v", err)
			}

			visitor := &TestVisitor{
				VisitLog: []string{},
			}

			input := tc.Input
			result, err := Walk(visitor, &input, tree)

			if err != nil {
				t.Fatalf("Walk returned error: %v", err)
			}

			output := map[string]interface{}{
				"result":   *result,
				"visitLog": visitor.VisitLog,
			}

			snapsConfig := snaps.WithConfig(snaps.JSON(snaps.JSONConfig{
				Width:    80,
				Indent:   "  ",
				SortKeys: true,
			}))
			snapsConfig.MatchJSON(t, output)
		})
	}
}
