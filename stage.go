// This file implements the compilation stage calculation algorithm
package patina

import (
	"fmt"
	"math"
	"sort"
	"strings"
)

const (
	// StageStatic represents the compilation stage for static definitions with no usages
	StageStatic Stage = 1
	// StageRuntime represents the runtime stage (maximum uint64 value)
	StageRuntime Stage = math.MaxUint64
)

// Definition with dependency count for topological sorting
type defWithDepCount struct {
	def  *Definition
	deps int
}

// CalculateStages calculates the compilation stages for all definitions in the given context.
// Returns an error if cyclic dependencies are detected.
func CalculateStages(ctx *Context) error {
	// First, collect all definitions in the context
	allDefs := collectAllDefinitions(ctx.Root)

	// Check for cyclic dependencies
	if cycle := detectCyclicDependencies(allDefs); cycle != nil {
		// Normalize the cycle to ensure consistent reporting
		// Find the alphabetically first name in the cycle
		firstIdx := 0
		for i, def := range cycle {
			if def.Name < cycle[firstIdx].Name {
				firstIdx = i
			}
		}

		// Rearrange the cycle to start with the alphabetically first name
		normalizedCycle := make([]*Definition, len(cycle))
		for i := range cycle {
			normalizedCycle[i] = cycle[(firstIdx+i)%len(cycle)]
		}

		// Create the error message with the normalized cycle
		cycleNames := make([]string, len(normalizedCycle))
		for i, def := range normalizedCycle {
			cycleNames[i] = def.Name
		}
		return fmt.Errorf("cyclic dependency detected: %s", strings.Join(cycleNames, " -> "))
	}

	// Sort definitions in topological order
	sortedDefs := sortDefinitionsTopologically(allDefs)

	// Calculate stages for all definitions
	calculateStagesForDefinitions(sortedDefs)

	return nil
}

// detectCyclicDependencies checks if there are any cyclic dependencies in the definitions
// Returns the cycle as a slice of definitions if found, or nil if no cycles exist
func detectCyclicDependencies(defs []*Definition) []*Definition {
	// Track visited and currently exploring definitions
	visited := make(map[*Definition]bool)
	exploring := make(map[*Definition]bool)

	// Check each definition for cycles
	for _, def := range defs {
		if cycle := detectCycleStartingFrom(def, visited, exploring, nil); cycle != nil {
			return cycle
		}
	}

	return nil
}

// detectCycleStartingFrom performs depth-first search to detect cycles
// Returns the cycle as a slice if found, or nil if no cycle
func detectCycleStartingFrom(def *Definition, visited, exploring map[*Definition]bool, path []*Definition) []*Definition {
	// Skip if already fully explored
	if visited[def] {
		return nil
	}

	// Cycle detected if we're revisiting a definition in the current exploration path
	if exploring[def] {
		// Find where in the path this definition appeared before
		for i, pathDef := range path {
			if pathDef == def {
				// Return the cycle (including the current definition again to show the loop)
				return append(path[i:], def)
			}
		}
		return path
	}

	// Mark as currently exploring
	exploring[def] = true
	path = append(path, def)

	// Recursively check usages
	for _, usage := range def.Usages {
		if cycle := detectCycleStartingFrom(usage, visited, exploring, path); cycle != nil {
			return cycle
		}
	}

	// Mark as fully visited and remove from exploring
	visited[def] = true
	exploring[def] = false

	return nil
}

// collectAllDefinitions gathers all definitions from the scope hierarchy
func collectAllDefinitions(scope *Scope) []*Definition {
	var defs []*Definition

	// Add all definitions in the current scope
	for _, def := range scope.defs {
		defs = append(defs, def)
	}

	// Recursively collect definitions from child scopes
	for _, child := range scope.children {
		defs = append(defs, collectAllDefinitions(child)...)
	}

	return defs
}

// sortDefinitionsTopologically sorts definitions based on their dependencies
func sortDefinitionsTopologically(defs []*Definition) []*Definition {
	// Create a map of definitions to their dependency counts
	defsWithCounts := make([]*defWithDepCount, 0, len(defs))
	for _, def := range defs {
		defsWithCounts = append(defsWithCounts, &defWithDepCount{
			def:  def,
			deps: len(def.Usages),
		})
	}

	// Sort by dependency count (ascending)
	sort.Slice(defsWithCounts, func(i, j int) bool {
		return defsWithCounts[i].deps < defsWithCounts[j].deps
	})

	// Extract sorted definitions
	sortedDefs := make([]*Definition, 0, len(defs))
	for _, defWithCount := range defsWithCounts {
		sortedDefs = append(sortedDefs, defWithCount.def)
	}

	return sortedDefs
}

// calculateStagesForDefinitions calculates the compilation stage for each definition
func calculateStagesForDefinitions(defs []*Definition) {
	// Create active and inactive sets
	inactive := make(map[*Definition]bool)
	active := make(map[*Definition]bool)

	// Initialize the inactive set with all definitions
	for _, def := range defs {
		inactive[def] = true
	}

	// Continue processing until the inactive set is empty
	for len(inactive) > 0 {
		progress := false

		// Iterate through definitions in the inactive set
		for def := range inactive {
			if canCalculateStage(def, inactive) {
				// Calculate the stage for this definition
				calculateDefinitionStage(def)

				// Move from inactive to active set
				delete(inactive, def)
				active[def] = true
				progress = true
			}
		}

		// If no progress was made in this iteration, there might be a cycle
		// In a real-world compiler, we'd handle this better, but for now
		// we'll just break any potential cycles by moving a definition
		if !progress && len(inactive) > 0 {
			// Choose any definition and force it to be calculated
			for def := range inactive {
				calculateDefinitionStage(def)
				delete(inactive, def)
				active[def] = true
				break
			}
		}
	}
}

// canCalculateStage checks if a definition's stage can be calculated
func canCalculateStage(def *Definition, inactive map[*Definition]bool) bool {
	// Check if any usages are still in the inactive set
	for _, usage := range def.Usages {
		if inactive[usage] {
			return false
		}
	}
	return true
}

// calculateDefinitionStage calculates the compilation stage for a single definition
func calculateDefinitionStage(def *Definition) {
	// If no usages and is static, assign StageStatic
	if len(def.Usages) == 0 {
		if def.IsStatic {
			def.Stage = StageStatic
		} else {
			def.Stage = StageRuntime
		}
		return
	}

	// Find maximum stage of all usages and add 1
	var maxUsageStage Stage
	for _, usage := range def.Usages {
		if usage.Stage > maxUsageStage {
			maxUsageStage = usage.Stage
		}
	}

	def.Stage = maxUsageStage + 1
}
