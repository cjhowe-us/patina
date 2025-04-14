// This file implements the compilation stage calculation algorithm
package patina

import (
	"math"
	"sort"
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
func CalculateStages(ctx *Context) {
	// First, collect all definitions in the context
	allDefs := collectAllDefinitions(ctx.Root)

	// Sort definitions in topological order
	sortedDefs := sortDefinitionsTopologically(allDefs)

	// Calculate stages for all definitions
	calculateStagesForDefinitions(sortedDefs)
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
			if canCalculateStage(def, active, inactive) {
				// Calculate the stage for this definition
				calculateDefinitionStage(def, active)

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
				calculateDefinitionStage(def, active)
				delete(inactive, def)
				active[def] = true
				break
			}
		}
	}
}

// canCalculateStage checks if a definition's stage can be calculated
func canCalculateStage(def *Definition, active, inactive map[*Definition]bool) bool {
	// Check if any usages are still in the inactive set
	for _, usage := range def.Usages {
		if inactive[usage] {
			return false
		}
	}
	return true
}

// calculateDefinitionStage calculates the compilation stage for a single definition
func calculateDefinitionStage(def *Definition, active map[*Definition]bool) {
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
