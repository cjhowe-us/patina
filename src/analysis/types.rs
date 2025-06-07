use crate::analysis::{Arena, Id, Stage};
use std::collections::HashMap;
use std::fmt;

// Helper to display Id<Type> more nicely
struct TypeId(Id<Type>);

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type{}", self.0.index())
    }
}

/// Type registry that manages all types in the system
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    types: Arena<Type>,
    type_cache: HashMap<String, Id<Type>>,
    builtins: HashMap<String, Id<Type>>,
}

/// Represents a type in the type system
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Integer type with sign and bit width
    Integer { signed: bool, bits: u16 },
    /// Floating point type with bit width
    Float { bits: u8 },
    /// Boolean type
    Boolean,
    /// String type
    String,
    /// Array type with element type and optional size
    Array {
        element: Id<Type>,
        size: Option<usize>,
    },
    /// Pointer type with target and modifier
    Pointer {
        target: Id<Type>,
        modifier: PointerModifier,
    },
    /// Function type with parameters and return type
    Function {
        params: Vec<Id<Type>>,
        return_type: Option<Id<Type>>,
    },
    /// Struct type with fields
    Struct { name: String, fields: Vec<Field> },
    /// Enum type with variants
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
    /// Type parameter for generics
    TypeParameter { name: String, index: usize },
    /// Generic type instantiation
    Generic { base: Id<Type>, args: Vec<Id<Type>> },
    /// Stage-aware code type (quoted expression type)
    Code { inner: Id<Type>, from_stage: Stage },
    /// Unknown type (for type inference)
    Unknown,
    /// Error type (for error recovery)
    Error,
}

/// Pointer type modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerModifier {
    None,
    Mutable,
    Owned,
    Raw,
}

/// Struct field
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Id<Type>,
    pub position: usize,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    pub discriminant: Option<i64>,
}

/// Stage-aware type that carries stage information
#[derive(Debug, Clone, PartialEq)]
pub struct StagedType {
    pub ty: Id<Type>,
    pub stage: Stage,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            types: Arena::new(),
            type_cache: HashMap::new(),
            builtins: HashMap::new(),
        };

        // Register builtin types
        registry.register_builtins();
        registry
    }

    fn register_builtins(&mut self) {
        // Integer types
        let i8_type = self.types.insert(Type::Integer {
            signed: true,
            bits: 8,
        });
        self.builtins.insert("i8".to_string(), i8_type);

        let i16_type = self.types.insert(Type::Integer {
            signed: true,
            bits: 16,
        });
        self.builtins.insert("i16".to_string(), i16_type);

        let i32_type = self.types.insert(Type::Integer {
            signed: true,
            bits: 32,
        });
        self.builtins.insert("i32".to_string(), i32_type);

        let i64_type = self.types.insert(Type::Integer {
            signed: true,
            bits: 64,
        });
        self.builtins.insert("i64".to_string(), i64_type);

        let u8_type = self.types.insert(Type::Integer {
            signed: false,
            bits: 8,
        });
        self.builtins.insert("u8".to_string(), u8_type);

        let u16_type = self.types.insert(Type::Integer {
            signed: false,
            bits: 16,
        });
        self.builtins.insert("u16".to_string(), u16_type);

        let u32_type = self.types.insert(Type::Integer {
            signed: false,
            bits: 32,
        });
        self.builtins.insert("u32".to_string(), u32_type);

        let u64_type = self.types.insert(Type::Integer {
            signed: false,
            bits: 64,
        });
        self.builtins.insert("u64".to_string(), u64_type);

        // Float types
        let f32_type = self.types.insert(Type::Float { bits: 32 });
        self.builtins.insert("f32".to_string(), f32_type);

        let f64_type = self.types.insert(Type::Float { bits: 64 });
        self.builtins.insert("f64".to_string(), f64_type);

        // Other basic types
        let bool_type = self.types.insert(Type::Boolean);
        self.builtins.insert("bool".to_string(), bool_type);

        let string_type = self.types.insert(Type::String);
        self.builtins.insert("string".to_string(), string_type);
    }

    pub fn get_builtin(&self, name: &str) -> Option<Id<Type>> {
        self.builtins.get(name).copied()
    }

    pub fn get_type(&self, id: Id<Type>) -> Option<&Type> {
        self.types.get(id)
    }

    pub fn insert_type(&mut self, ty: Type) -> Id<Type> {
        // Check cache for structural types
        if let Some(cache_key) = self.type_cache_key(&ty) {
            if let Some(&cached_id) = self.type_cache.get(&cache_key) {
                return cached_id;
            }
            let id = self.types.insert(ty);
            self.type_cache.insert(cache_key, id);
            id
        } else {
            self.types.insert(ty)
        }
    }

    fn type_cache_key(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Array { element, size } => Some(format!("[{element:?}; {size:?}]")),
            Type::Pointer { target, modifier } => Some(format!("*{modifier:?} {target:?}")),
            Type::Function {
                params,
                return_type,
            } => Some(format!("fn({params:?}) -> {return_type:?}")),
            _ => None,
        }
    }

    pub fn make_array_type(&mut self, element: Id<Type>, size: Option<usize>) -> Id<Type> {
        self.insert_type(Type::Array { element, size })
    }

    pub fn make_pointer_type(&mut self, target: Id<Type>, modifier: PointerModifier) -> Id<Type> {
        self.insert_type(Type::Pointer { target, modifier })
    }

    pub fn make_function_type(
        &mut self,
        params: Vec<Id<Type>>,
        return_type: Option<Id<Type>>,
    ) -> Id<Type> {
        self.insert_type(Type::Function {
            params,
            return_type,
        })
    }

    pub fn make_code_type(&mut self, inner: Id<Type>, from_stage: Stage) -> Id<Type> {
        self.insert_type(Type::Code { inner, from_stage })
    }

    pub fn is_code_type(&self, ty_id: Id<Type>) -> bool {
        matches!(self.get_type(ty_id), Some(Type::Code { .. }))
    }

    pub fn unwrap_code_type(&self, ty_id: Id<Type>) -> Option<(Id<Type>, Stage)> {
        match self.get_type(ty_id) {
            Some(Type::Code { inner, from_stage }) => Some((*inner, *from_stage)),
            _ => None,
        }
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer { signed, bits } => {
                write!(f, "{}{}", if *signed { "i" } else { "u" }, bits)
            }
            Type::Float { bits } => write!(f, "f{bits}"),
            Type::Boolean => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Array { element, size } => match size {
                Some(s) => write!(f, "[{}; {}]", TypeId(*element), s),
                None => write!(f, "[{}]", TypeId(*element)),
            },
            Type::Pointer { target, modifier } => {
                let mod_str = match modifier {
                    PointerModifier::None => "",
                    PointerModifier::Mutable => "mut ",
                    PointerModifier::Owned => "own ",
                    PointerModifier::Raw => "raw ",
                };
                write!(f, "*{}{}", mod_str, TypeId(*target))
            }
            Type::Function {
                params,
                return_type,
            } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", TypeId(*param))?;
                }
                write!(f, ")")?;
                if let Some(ret) = return_type {
                    write!(f, " -> {}", TypeId(*ret))?;
                }
                Ok(())
            }
            Type::Struct { name, .. } => write!(f, "struct {name}"),
            Type::Enum { name, .. } => write!(f, "enum {name}"),
            Type::TypeParameter { name, index } => write!(f, "{name} #{index}"),
            Type::Generic { base, args } => {
                write!(f, "{}<", TypeId(*base))?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", TypeId(*arg))?;
                }
                write!(f, ">")
            }
            Type::Code { inner, from_stage } => {
                write!(f, "Code<{}@{}>", TypeId(*inner), from_stage)
            }
            Type::Unknown => write!(f, "?"),
            Type::Error => write!(f, "!"),
        }
    }
}

/// Type checking context for stage-aware type checking
pub struct TypeChecker<'a> {
    registry: &'a mut TypeRegistry,
    current_stage: Stage,
}

impl<'a> TypeChecker<'a> {
    pub fn new(registry: &'a mut TypeRegistry) -> Self {
        Self {
            registry,
            current_stage: Stage::RUNTIME,
        }
    }

    pub fn with_stage(mut self, stage: Stage) -> Self {
        self.current_stage = stage;
        self
    }

    pub fn check_splice(
        &self,
        expr_type: Id<Type>,
        splice_stage: Stage,
    ) -> Result<Id<Type>, TypeError> {
        match self.registry.unwrap_code_type(expr_type) {
            Some((inner_type, from_stage)) => {
                // Splice must come from previous stage
                match splice_stage.prev() {
                    Some(prev_stage) if prev_stage == from_stage => Ok(inner_type),
                    _ => Err(TypeError::InvalidSplice {
                        expected_stage: splice_stage.prev().unwrap_or(Stage::COMPILE_TIME),
                        actual_stage: from_stage,
                    }),
                }
            }
            None => Err(TypeError::NotCodeType { ty: expr_type }),
        }
    }

    pub fn check_quote(&mut self, inner_type: Id<Type>, quote_stage: Stage) -> Id<Type> {
        self.registry.make_code_type(inner_type, quote_stage)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    InvalidSplice {
        expected_stage: Stage,
        actual_stage: Stage,
    },
    NotCodeType {
        ty: Id<Type>,
    },
    TypeMismatch {
        expected: Id<Type>,
        actual: Id<Type>,
    },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::InvalidSplice {
                expected_stage,
                actual_stage,
            } => {
                write!(
                    f,
                    "Invalid splice: expected code from {expected_stage}, but got code from {actual_stage}"
                )
            }
            TypeError::NotCodeType { ty } => {
                write!(f, "Cannot splice non-code type: {ty:?}")
            }
            TypeError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {expected:?}, got {actual:?}")
            }
        }
    }
}

impl std::error::Error for TypeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_registry_creation() {
        let registry = TypeRegistry::new();

        // Check builtin types
        assert!(registry.get_builtin("i32").is_some());
        assert!(registry.get_builtin("bool").is_some());
        assert!(registry.get_builtin("string").is_some());
        assert!(registry.get_builtin("unknown").is_none());
    }

    #[test]
    fn test_builtin_types() {
        let registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let i32_info = registry.get_type(i32_type).unwrap();
        assert_eq!(
            *i32_info,
            Type::Integer {
                signed: true,
                bits: 32
            }
        );

        let bool_type = registry.get_builtin("bool").unwrap();
        let bool_info = registry.get_type(bool_type).unwrap();
        assert_eq!(*bool_info, Type::Boolean);
    }

    #[test]
    fn test_array_type() {
        let mut registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let array_type = registry.make_array_type(i32_type, Some(10));

        let array_info = registry.get_type(array_type).unwrap();
        assert_eq!(
            *array_info,
            Type::Array {
                element: i32_type,
                size: Some(10)
            }
        );

        // Test caching
        let array_type2 = registry.make_array_type(i32_type, Some(10));
        assert_eq!(array_type, array_type2);
    }

    #[test]
    fn test_pointer_type() {
        let mut registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let ptr_type = registry.make_pointer_type(i32_type, PointerModifier::Mutable);

        let ptr_info = registry.get_type(ptr_type).unwrap();
        assert_eq!(
            *ptr_info,
            Type::Pointer {
                target: i32_type,
                modifier: PointerModifier::Mutable
            }
        );
    }

    #[test]
    fn test_function_type() {
        let mut registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let bool_type = registry.get_builtin("bool").unwrap();

        let func_type = registry.make_function_type(vec![i32_type, i32_type], Some(bool_type));

        let func_info = registry.get_type(func_type).unwrap();
        assert_eq!(
            *func_info,
            Type::Function {
                params: vec![i32_type, i32_type],
                return_type: Some(bool_type)
            }
        );
    }

    #[test]
    fn test_code_type() {
        let mut registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let code_type = registry.make_code_type(i32_type, Stage::COMPILE_TIME);

        assert!(registry.is_code_type(code_type));
        assert!(!registry.is_code_type(i32_type));

        let (inner, stage) = registry.unwrap_code_type(code_type).unwrap();
        assert_eq!(inner, i32_type);
        assert_eq!(stage, Stage::COMPILE_TIME);
    }

    #[test]
    fn test_type_display() {
        let registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let i32_info = registry.get_type(i32_type).unwrap();
        assert_eq!(format!("{}", i32_info), "i32");

        let array_type = Type::Array {
            element: i32_type,
            size: Some(5),
        };
        assert_eq!(format!("{}", array_type), "[Type2; 5]");

        let ptr_type = Type::Pointer {
            target: i32_type,
            modifier: PointerModifier::Owned,
        };
        assert_eq!(format!("{}", ptr_type), "*own Type2");
    }

    #[test]
    fn test_type_checker_splice() {
        let mut registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let code_type = registry.make_code_type(i32_type, Stage::COMPILE_TIME);

        let checker = TypeChecker::new(&mut registry);

        // Valid splice at stage 1
        let result = checker.check_splice(code_type, Stage::RUNTIME);
        assert_eq!(result, Ok(i32_type));

        // Invalid splice at stage 0
        let result = checker.check_splice(code_type, Stage::COMPILE_TIME);
        assert!(matches!(result, Err(TypeError::InvalidSplice { .. })));

        // Not a code type
        let result = checker.check_splice(i32_type, Stage::RUNTIME);
        assert!(matches!(result, Err(TypeError::NotCodeType { .. })));
    }

    #[test]
    fn test_type_checker_quote() {
        let mut registry = TypeRegistry::new();

        let i32_type = registry.get_builtin("i32").unwrap();
        let quoted = {
            let mut checker = TypeChecker::new(&mut registry);
            checker.check_quote(i32_type, Stage::RUNTIME)
        };

        let quoted_info = registry.get_type(quoted).unwrap();
        assert_eq!(
            *quoted_info,
            Type::Code {
                inner: i32_type,
                from_stage: Stage::RUNTIME
            }
        );
    }
}
