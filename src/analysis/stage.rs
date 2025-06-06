use std::fmt;

/// Represents a compile-time stage level.
/// Stage 0 is compile-time, Stage 1 is runtime, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stage(pub u32);

impl Stage {
    /// Compile-time stage (macros, const evaluation)
    pub const COMPILE_TIME: Stage = Stage(0);

    /// Runtime stage (normal program execution)
    pub const RUNTIME: Stage = Stage(1);

    /// Creates a new stage with the given level
    pub fn new(level: u32) -> Self {
        Stage(level)
    }

    /// Returns the stage level as a u32
    pub fn level(self) -> u32 {
        self.0
    }

    /// Returns the next stage (n + 1)
    pub fn next(self) -> Stage {
        Stage(self.0.saturating_add(1))
    }

    /// Returns the previous stage if it exists
    pub fn prev(self) -> Option<Stage> {
        if self.0 > 0 {
            Some(Stage(self.0 - 1))
        } else {
            None
        }
    }

    /// Checks if this is the compile-time stage (stage 0)
    pub fn is_compile_time(self) -> bool {
        self.0 == 0
    }

    /// Checks if this is the runtime stage (stage 1)
    pub fn is_runtime(self) -> bool {
        self.0 == 1
    }

    /// Checks if a symbol at `symbol_stage` is visible at `self` stage.
    /// Symbols from earlier stages are visible at later stages.
    pub fn can_see_symbol_at(self, symbol_stage: Stage) -> bool {
        symbol_stage.0 <= self.0
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stage {}", self.0)
    }
}

impl From<u32> for Stage {
    fn from(level: u32) -> Self {
        Stage(level)
    }
}

impl Default for Stage {
    fn default() -> Self {
        Stage::RUNTIME
    }
}

/// Error type for stage-related operations
#[derive(Debug, Clone, PartialEq)]
pub enum StageError {
    /// Attempted to get previous stage of stage 0
    NoPreviousStage,
    /// Invalid stage transition
    InvalidTransition { from: Stage, to: Stage },
    /// Symbol not visible at current stage
    SymbolNotVisible {
        symbol_stage: Stage,
        current_stage: Stage,
    },
    /// Invalid splice operation
    InvalidSplice {
        expr_stage: Stage,
        current_stage: Stage,
    },
    /// Invalid quote operation
    InvalidQuote { current_stage: Stage },
}

impl fmt::Display for StageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StageError::NoPreviousStage => {
                write!(f, "Cannot get previous stage of stage 0")
            }
            StageError::InvalidTransition { from, to } => {
                write!(f, "Invalid stage transition from {from} to {to}")
            }
            StageError::SymbolNotVisible {
                symbol_stage,
                current_stage,
            } => {
                write!(
                    f,
                    "Symbol at {symbol_stage} is not visible at {current_stage}"
                )
            }
            StageError::InvalidSplice {
                expr_stage,
                current_stage,
            } => {
                write!(
                    f,
                    "Cannot splice expression from {expr_stage} at {current_stage}"
                )
            }
            StageError::InvalidQuote { current_stage } => {
                write!(f, "Invalid quote operation at {current_stage}")
            }
        }
    }
}

impl std::error::Error for StageError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_creation() {
        let stage0 = Stage::COMPILE_TIME;
        let stage1 = Stage::RUNTIME;
        let stage2 = Stage::new(2);

        assert_eq!(stage0.level(), 0);
        assert_eq!(stage1.level(), 1);
        assert_eq!(stage2.level(), 2);
    }

    #[test]
    fn test_stage_comparison() {
        let stage0 = Stage::COMPILE_TIME;
        let stage1 = Stage::RUNTIME;
        let stage2 = Stage::new(2);

        assert!(stage0 < stage1);
        assert!(stage1 < stage2);
        assert!(stage0 < stage2);
    }

    #[test]
    fn test_stage_next() {
        let stage0 = Stage::COMPILE_TIME;
        let stage1 = stage0.next();
        let stage2 = stage1.next();

        assert_eq!(stage1, Stage::RUNTIME);
        assert_eq!(stage2, Stage::new(2));
    }

    #[test]
    fn test_stage_prev() {
        let stage0 = Stage::COMPILE_TIME;
        let stage1 = Stage::RUNTIME;
        let stage2 = Stage::new(2);

        assert_eq!(stage0.prev(), None);
        assert_eq!(stage1.prev(), Some(Stage::COMPILE_TIME));
        assert_eq!(stage2.prev(), Some(Stage::RUNTIME));
    }

    #[test]
    fn test_stage_predicates() {
        let stage0 = Stage::COMPILE_TIME;
        let stage1 = Stage::RUNTIME;
        let stage2 = Stage::new(2);

        assert!(stage0.is_compile_time());
        assert!(!stage0.is_runtime());

        assert!(!stage1.is_compile_time());
        assert!(stage1.is_runtime());

        assert!(!stage2.is_compile_time());
        assert!(!stage2.is_runtime());
    }

    #[test]
    fn test_symbol_visibility() {
        let stage0 = Stage::COMPILE_TIME;
        let stage1 = Stage::RUNTIME;
        let stage2 = Stage::new(2);

        // Stage 0 can only see stage 0 symbols
        assert!(stage0.can_see_symbol_at(stage0));
        assert!(!stage0.can_see_symbol_at(stage1));
        assert!(!stage0.can_see_symbol_at(stage2));

        // Stage 1 can see stage 0 and 1 symbols
        assert!(stage1.can_see_symbol_at(stage0));
        assert!(stage1.can_see_symbol_at(stage1));
        assert!(!stage1.can_see_symbol_at(stage2));

        // Stage 2 can see stages 0, 1, and 2 symbols
        assert!(stage2.can_see_symbol_at(stage0));
        assert!(stage2.can_see_symbol_at(stage1));
        assert!(stage2.can_see_symbol_at(stage2));
    }

    #[test]
    fn test_stage_display() {
        assert_eq!(format!("{}", Stage::COMPILE_TIME), "stage 0");
        assert_eq!(format!("{}", Stage::RUNTIME), "stage 1");
        assert_eq!(format!("{}", Stage::new(42)), "stage 42");
    }

    #[test]
    fn test_stage_from_u32() {
        let stage: Stage = 5.into();
        assert_eq!(stage, Stage::new(5));
    }

    #[test]
    fn test_stage_default() {
        let stage = Stage::default();
        assert_eq!(stage, Stage::RUNTIME);
    }

    #[test]
    fn test_stage_error_display() {
        let err = StageError::NoPreviousStage;
        assert_eq!(format!("{}", err), "Cannot get previous stage of stage 0");

        let err = StageError::InvalidTransition {
            from: Stage::RUNTIME,
            to: Stage::COMPILE_TIME,
        };
        assert_eq!(
            format!("{}", err),
            "Invalid stage transition from stage 1 to stage 0"
        );

        let err = StageError::SymbolNotVisible {
            symbol_stage: Stage::new(2),
            current_stage: Stage::RUNTIME,
        };
        assert_eq!(
            format!("{}", err),
            "Symbol at stage 2 is not visible at stage 1"
        );
    }

    #[test]
    fn test_stage_saturating_add() {
        let max_stage = Stage(u32::MAX);
        let next = max_stage.next();
        assert_eq!(next.level(), u32::MAX); // Should saturate, not overflow
    }
}
