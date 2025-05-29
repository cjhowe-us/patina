# Error Handling Improvements

## Current Issues

### Mutex Lock Unwrapping
- **Location**: `src/analysis/context.rs`
- **Issue**: Context methods use `.unwrap()` on mutex locks
- **Risk**: Could panic if a thread panics while holding a lock
- **Priority**: Medium

## Proposed Solutions

### 1. Use expect() with Descriptive Messages
```rust
// Instead of:
let guard = self.mutex.lock().unwrap();

// Use:
let guard = self.mutex.lock()
    .expect("Failed to acquire mutex lock in Context");
```

### 2. Return Result Types
```rust
// Change method signatures to return Result
pub fn add_symbol(&self, symbol: Symbol) -> Result<(), AnalysisError> {
    let mut symbols = self.symbols.lock()
        .map_err(|_| AnalysisError::LockPoisoned)?;
    symbols.insert(symbol.name.clone(), symbol);
    Ok(())
}
```

### 3. Create Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Mutex lock poisoned")]
    LockPoisoned,
    
    #[error("Symbol '{0}' already defined")]
    SymbolRedefinition(String),
    
    // ... other error variants
}
```

## Implementation Plan

1. **Phase 1**: Replace `.unwrap()` with `.expect()` 
   - Quick fix to provide better panic messages
   - No API changes required

2. **Phase 2**: Design comprehensive error types
   - Create `AnalysisError` enum
   - Add context-specific error variants

3. **Phase 3**: Update method signatures
   - Change return types to `Result<T, AnalysisError>`
   - Update all call sites

4. **Phase 4**: Error recovery
   - Implement graceful error handling
   - Continue analysis after non-fatal errors

## Benefits

- Better debugging when failures occur
- Possibility of error recovery
- More robust multi-threaded usage
- Clearer error messages for users

## Files to Update

- `src/analysis/context.rs`
- `src/analysis/symbol_table.rs`
- `src/analysis/stage.rs`
- Any other files using `.unwrap()` on locks