pub mod arena;
pub mod context;
pub mod diagnostics;
pub mod id;
pub mod stage;
pub mod symbol_table;
pub mod types;

pub use arena::Arena;
pub use context::AnalysisContext;
pub use diagnostics::{Diagnostic, DiagnosticLevel, ErrorCollector, Note, Suggestion};
pub use id::{Id, ScopeId, SymbolId, TypeId};
pub use stage::{Stage, StageError};
pub use symbol_table::{ParamInfo, Scope, ScopeKind, Symbol, SymbolError, SymbolKind, SymbolTable, TypeDefinition};
pub use types::{Field, PointerModifier, StagedType, Type, TypeChecker, TypeError, TypeRegistry, Variant};