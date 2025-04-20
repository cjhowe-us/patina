package dependency

import (
	"github.com/cjhowedev/patina/scope"
)

// DependencyGraph stores the dependencies between definitions.
// Key: A definition. Value: A list of definitions it depends on.
type DependencyGraph map[*scope.Definition][]*scope.Definition

// AddDependency records that 'def' depends on 'dependsOn'.
func (g DependencyGraph) AddDependency(def, dependsOn *scope.Definition) {
	if def == nil || dependsOn == nil || def == dependsOn {
		return // Ignore self-dependencies or nil
	}
	// Ensure the map entry exists
	if _, ok := g[def]; !ok {
		g[def] = []*scope.Definition{}
	}
	// Avoid duplicate dependencies
	for _, existing := range g[def] {
		if existing == dependsOn {
			return
		}
	}
	g[def] = append(g[def], dependsOn)
}
