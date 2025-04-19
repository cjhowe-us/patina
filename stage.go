package patina

import (
	"fmt"
	"math"
	"sort"
	"strings"
)

const (
	StageStatic  Stage = 1
	StageRuntime Stage = math.MaxUint64
)

type defWithDepCount struct {
	def  *Definition
	deps int
}

func CalculateStages(ctx *Context) error {
	allDefs := collectAllDefinitions(ctx.Root)
	removeSelfReferences(allDefs)
	if cycle := detectCyclicDependencies(allDefs); cycle != nil {
		firstIdx := 0
		for i, def := range cycle {
			if def.Name < cycle[firstIdx].Name {
				firstIdx = i
			}
		}
		normalizedCycle := make([]*Definition, len(cycle))
		for i := range cycle {
			normalizedCycle[i] = cycle[(firstIdx+i)%len(cycle)]
		}
		cycleNames := make([]string, len(normalizedCycle))
		for i, def := range normalizedCycle {
			cycleNames[i] = def.Name
		}
		if len(normalizedCycle) > 0 {
			cycleNames = append(cycleNames, normalizedCycle[0].Name)
		}
		return fmt.Errorf("cyclic dependency detected: %s", strings.Join(cycleNames, " -> "))
	}
	sortedDefs := sortDefinitionsTopologically(allDefs)
	calculateStagesForDefinitions(sortedDefs)
	return nil
}

func removeSelfReferences(defs []*Definition) {
	selfRefOnly := make(map[*Definition]bool)
	for _, def := range defs {
		if len(def.Usages) == 1 && def.Usages[0] == def {
			selfRefOnly[def] = true
		}
	}
	for _, def := range defs {
		if selfRefOnly[def] {
			continue
		}
		filteredUsages := make([]*Definition, 0, len(def.Usages))
		for _, usage := range def.Usages {
			if usage != def {
				filteredUsages = append(filteredUsages, usage)
			}
		}
		def.Usages = filteredUsages
	}
}

func detectCyclicDependencies(defs []*Definition) []*Definition {
	for _, def := range defs {
		for _, usage := range def.Usages {
			if usage == def {
				return []*Definition{def, def}
			}
		}
	}
	visited := make(map[*Definition]bool)
	exploring := make(map[*Definition]bool)
	for _, def := range defs {
		if cycle := detectCycleStartingFrom(def, visited, exploring, nil); cycle != nil {
			return cycle
		}
	}
	return nil
}

func detectCycleStartingFrom(def *Definition, visited, exploring map[*Definition]bool, path []*Definition) []*Definition {
	if visited[def] {
		return nil
	}
	if exploring[def] {
		for i, pathDef := range path {
			if pathDef == def {
				cycle := path[i:]
				return cycle
			}
		}
		return nil
	}
	exploring[def] = true
	path = append(path, def)
	for _, usage := range def.Usages {
		if cycle := detectCycleStartingFrom(usage, visited, exploring, path); cycle != nil {
			return cycle
		}
	}
	visited[def] = true
	exploring[def] = false
	return nil
}

func collectAllDefinitions(scope *Scope) []*Definition {
	var defs []*Definition
	for _, def := range scope.defs {
		defs = append(defs, def)
	}
	for _, child := range scope.children {
		defs = append(defs, collectAllDefinitions(child)...)
	}
	return defs
}

func sortDefinitionsTopologically(defs []*Definition) []*Definition {
	defsWithCounts := make([]*defWithDepCount, 0, len(defs))
	for _, def := range defs {
		defsWithCounts = append(defsWithCounts, &defWithDepCount{
			def:  def,
			deps: len(def.Usages),
		})
	}
	sort.Slice(defsWithCounts, func(i, j int) bool {
		return defsWithCounts[i].deps < defsWithCounts[j].deps
	})
	sortedDefs := make([]*Definition, 0, len(defs))
	for _, defWithCount := range defsWithCounts {
		sortedDefs = append(sortedDefs, defWithCount.def)
	}
	return sortedDefs
}

func calculateStagesForDefinitions(defs []*Definition) {
	inactive := make(map[*Definition]bool)
	active := make(map[*Definition]bool)
	for _, def := range defs {
		inactive[def] = true
	}
	for len(inactive) > 0 {
		progress := false
		for def := range inactive {
			if canCalculateStage(def, inactive) {
				calculateDefinitionStage(def)
				delete(inactive, def)
				active[def] = true
				progress = true
			}
		}
		if !progress && len(inactive) > 0 {
			for def := range inactive {
				calculateDefinitionStage(def)
				delete(inactive, def)
				active[def] = true
				break
			}
		}
	}
}

func canCalculateStage(def *Definition, inactive map[*Definition]bool) bool {
	for _, usage := range def.Usages {
		if inactive[usage] {
			return false
		}
	}
	return true
}

func calculateDefinitionStage(def *Definition) {
	if len(def.Usages) == 0 {
		if def.IsStatic {
			def.Stage = StageStatic
		} else {
			def.Stage = StageRuntime
		}
		return
	}
	var maxUsageStage Stage
	for _, usage := range def.Usages {
		if usage.Stage > maxUsageStage {
			maxUsageStage = usage.Stage
		}
	}
	def.Stage = maxUsageStage + 1
}
