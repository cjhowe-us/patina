// Package ast provides the abstract syntax tree representation for the Patina language.
package ast

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

func TestMain(m *testing.M) {
	v := m.Run()

	snaps.Clean(m, snaps.CleanOpts{Sort: true})

	os.Exit(v)
}

var snapshotExt = snaps.Ext(".json")

type TestError struct {
	Error string `json:"error"`
}

func NewTestError(err error) TestError {
	return TestError{Error: err.Error()}
}

// RunTests runs tests for each test case and compares results with snapshots.
// It accepts either a single filename or multiple filenames.
func RunTests[I, O any](
	t *testing.T,
	filenames []string,
	runTest func(I, string) (O, error),
) {
	t.Helper()

	var allTestCases []TestCase[I]

	for _, filename := range filenames {
		testCases, err := loadTestCases[I](filename)
		if err != nil {
			t.Fatalf("Failed to load test cases from %s: %v", filename, err)
		}
		allTestCases = append(allTestCases, testCases...)
	}

	for _, tc := range allTestCases {
		t.Run(tc.Name, func(t *testing.T) {
			nameNoSpaces := strings.ReplaceAll(tc.Name, " ", "_")
			output, err := runTest(tc.Input, nameNoSpaces)

			testName := strings.SplitN(t.Name(), "/", 2)[0]
			snapshotDir := snaps.Dir(fmt.Sprintf("testing/snapshots/%s", testName))
			snapsConfig := snaps.WithConfig(snapshotExt, snapshotDir, snaps.Filename(nameNoSpaces))
			if err == nil {
				snapsConfig.MatchStandaloneJSON(t, output)
			} else {
				snapsConfig.MatchStandaloneJSON(t, NewTestError(err))
			}
		})
	}
}

// TestCase represents a generic test case with a name and source code.
type TestCase[Input any] struct {
	Name  string `json:"name"`
	Input Input  `json:"input"`
}

func loadTestCases[I any](filename string) ([]TestCase[I], error) {
	fileData, err := os.ReadFile(filepath.Join("testing/cases", filename))
	if err != nil {
		return nil, err
	}

	var testCases []TestCase[I]
	if err := json.Unmarshal(fileData, &testCases); err != nil {
		return nil, err
	}

	return testCases, nil
}
