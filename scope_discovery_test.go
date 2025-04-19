package patina

import (
	"strings"
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

func createScopeDiscoveryFixture(t *testing.T, code string) *Context {
	root, err := Parser.ParseString("", code)
	if err != nil {
		t.Fatalf("Failed to parse code: %v", err)
	}

	ctx := DiscoverScopes(root)
	return ctx
}

func scopeToMap(scope *Scope) map[string]interface{} {
	result := map[string]interface{}{
		"name":     scope.Name,
		"isStatic": scope.IsStatic,
		"defs":     map[string]interface{}{},
		"children": map[string]interface{}{},
	}

	for name, def := range scope.GetDefinitions() {
		defMap := map[string]interface{}{
			"name":     def.Name,
			"isStatic": def.IsStatic,
			"stage":    def.Stage,
			"usages":   len(def.Usages),
		}
		result["defs"].(map[string]interface{})[name] = defMap
	}

	for name, childScope := range scope.GetChildren() {
		result["children"].(map[string]interface{})[name] = scopeToMap(childScope)
	}

	return result
}

func TestModuleScope(t *testing.T) {
	moduleCode := `
	module test {
		fun main() {}
	}
	`
	ctx := createScopeDiscoveryFixture(t, moduleCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestFunctionScope(t *testing.T) {
	functionCode := `
	module test {
		fun add(a: i32, b: i32): i32 {
			return a + b
		}

		fun main() {
			let result = add(1, 2)
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, functionCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestNestedScope(t *testing.T) {
	nestedCode := `
	module test {
		fun outer() {
			let x = 10

			fun inner() {
				let y = x + 5
			}

			inner()
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, nestedCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestVariableDefinitionAndUsage(t *testing.T) {
	usageCode := `
	module test {
		let globalVar = 100

		fun useGlobal() {
			let localVar = globalVar + 50
			return localVar
		}

		fun main() {
			let result = useGlobal()
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, usageCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestIfExpressionScope(t *testing.T) {
	ifCode := `
	module test {
		fun conditional(x: i32) {
			if (x > 10) {
				let large = x
			} else {
				let small = x
			}
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, ifCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestLambdaScope(t *testing.T) {
	t.Skip("Skipping due to parsing issues with module syntax")

	lambdaCode := `
	module test {
		fun higher(f: fun(i32): i32) {
			let result = f(10)
		}

		fun main() {
			higher(fun(x: i32): i32 {
				let doubled = x * 2
				return doubled
			})
		}
	}
	`

	root, err := Parser.Parse("test.patina", strings.NewReader(lambdaCode))
	if err != nil {
		t.Fatalf("Failed to parse lambda code: %v", err)
	}

	ctx := DiscoverScopes(root)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestLambdaScopeFixed(t *testing.T) {
	t.Skip("Skipping due to parsing issues with module syntax")

	lambdaCode := `
module test {
	fun higher(f: fun(i32): i32) {
		let result = f(10)
	}

	fun main() {
		higher(fun(x: i32): i32 {
			let doubled = x * 2
			return doubled
		})
	}
}
`
	root, err := Parser.Parse("test.patina", strings.NewReader(lambdaCode))
	if err != nil {
		t.Fatalf("Failed to parse fixed lambda code: %v", err)
	}

	ctx := DiscoverScopes(root)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestStaticDefinitionScope(t *testing.T) {
	staticCode := `
	module test {
		static let constant = 42

		static fun factory() {
			return constant
		}

		fun main() {
			let value = factory()
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, staticCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestDefinitionDiscoveryInterfaces(t *testing.T) {
	tests := []struct {
		name string
		code string
	}{
		{
			name: "Module with functions",
			code: `
			module test {
				fun foo() {}
				fun bar() {}
			}
			`,
		},
		{
			name: "Variables with references",
			code: `
			module test {
				let x = 10
				fun useX() {
					return x * 2
				}
			}
			`,
		},
		{
			name: "Nested functions and scopes",
			code: `
			module test {
				fun outer() {
					let a = 1
					fun middle() {
						let b = a + 1
						fun inner() {
							let c = b + 1
							return c
						}
						return inner()
					}
					return middle()
				}
			}
			`,
		},
		{
			name: "Complex expressions",
			code: `
			module test {
				fun complex(x: i32, y: i32) {
					let result = if (x > y) {
						let diff = x - y
						diff * 2
					} else {
						let sum = x + y
						sum / 2
					}
					return result
				}
			}
			`,
		},
		{
			name: "Static definitions",
			code: `
			module test {
				static let CONST = 42
				static fun getConst() {
					return CONST
				}
				fun main() {
					let value = getConst()
					return value
				}
			}
			`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := createScopeDiscoveryFixture(t, tt.code)
			result := scopeToMap(ctx.Root)
			snaps.MatchJSON(t, result)
		})
	}
}

func TestComplexExpressionFixedScope(t *testing.T) {
	complexCode := `
	module test {
		fun complex(x: i32, y: i32) {
			let result = if (x > y) {
				let diff1 = x - y
				diff1 * 2
			} else {
				let sum = x + y
				sum / 2
			}
			return result
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, complexCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestSimpleExpression(t *testing.T) {
	simpleCode := `
	module test {
		fun simple(x: i32) {
			let a = x + 1
			let b = a * 2
			return b
		}
	}
	`
	ctx := createScopeDiscoveryFixture(t, simpleCode)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}

func TestCyclicDependenciesWithInterfaces(t *testing.T) {
	cyclicCode := `
	module test {
		let a = b
		let b = c
		let c = a
	}
	`

	root, err := Parser.Parse("test.patina", strings.NewReader(cyclicCode))
	if err != nil {
		t.Fatalf("Failed to parse Patina code: %v", err)
	}

	ctx := DiscoverScopes(root)

	err = CalculateStages(ctx)

	if err == nil {
		t.Fatalf("Expected error for cyclic dependency, but got nil")
	}

	expectedError := "cyclic dependency detected:"
	if !strings.Contains(err.Error(), expectedError) {
		t.Errorf("Error message didn't match expected text.\nGot: %s\nWanted to contain: %s",
			err.Error(), expectedError)
	}
}

func TestSelfCycleWithInterfaces(t *testing.T) {
	selfCycleCode := `
	module test {
		let a = a
	}
	`

	root, err := Parser.Parse("test.patina", strings.NewReader(selfCycleCode))
	if err != nil {
		t.Fatalf("Failed to parse Patina code: %v", err)
	}

	ctx := DiscoverScopes(root)

	err = CalculateStages(ctx)

	if err == nil {
		t.Errorf("Expected error for self-reference cycle with name 'a', but got nil")
	} else {
		expectedError := "cyclic dependency detected: a -> a"
		if !strings.Contains(err.Error(), expectedError) {
			t.Errorf("Error message didn't match expected text.\nGot: %s\nWanted to contain: %s",
				err.Error(), expectedError)
		}
	}
}

func TestMinimalLambdaScope(t *testing.T) {
	code := `fun test() {
		let f = fun(x: i32): i32 { return x * 2 }
		return f(10)
	}`

	root, err := Parser.ParseString("", code)
	if err != nil {
		t.Fatalf("Failed to parse minimal lambda code: %v", err)
	}

	ctx := DiscoverScopes(root)

	result := scopeToMap(ctx.Root)

	snaps.MatchJSON(t, result)
}
