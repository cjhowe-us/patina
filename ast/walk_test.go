package ast

import (
	"encoding/json"
	"reflect"
	"testing"
)

type VisitLog struct {
	Type    string          `json:"type"`
	Message json.RawMessage `json:"message"`
	IsEnter bool            `json:"isEnter"`
}

type LogVisitor struct {
	Log []VisitLog `json:"log"`
}

func (v *LogVisitor) Enter(node SyntaxNode, request *string) ([]*string, error) {
	jsonNode, err := json.Marshal(node)
	if err != nil {
		return nil, err
	}
	v.Log = append(v.Log, VisitLog{Type: reflect.TypeOf(node).String(), Message: jsonNode, IsEnter: true})
	return nil, nil
}

func (v *LogVisitor) Exit(node SyntaxNode, childrenOutputs []*string) (*string, error) {
	jsonNode, err := json.Marshal(node)
	if err != nil {
		return nil, err
	}
	v.Log = append(v.Log, VisitLog{Type: reflect.TypeOf(node).String(), Message: jsonNode, IsEnter: false})
	return nil, nil
}

// WalkTestOutput holds the output from a walk AST test.
type WalkTestOutput struct {
	Result  *string    `json:"result"`
	Visitor LogVisitor `json:"visitor"`
}

func TestWalk(t *testing.T) {
	RunTests(
		t,
		[]string{"walk.json", "readme_examples.json"},
		func(input string, name string) (WalkTestOutput, error) {
			root, err := Parser.ParseString(name, input)
			if err != nil || root == nil {
				return WalkTestOutput{}, err
			}
			visitor := LogVisitor{Log: []VisitLog{}}
			result, err := Walk(&visitor, new(string), root)
			if err != nil {
				return WalkTestOutput{}, err
			}
			return WalkTestOutput{Result: result, Visitor: visitor}, nil
		},
	)
}
