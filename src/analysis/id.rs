use std::fmt::{self, Display};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Id<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

// Manually implement Clone for Id<T> regardless of whether T is Clone
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// Manually implement Copy for Id<T> regardless of whether T is Copy
impl<T> Copy for Id<T> {}

impl<T> Id<T> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }

    pub fn index(self) -> usize {
        self.index
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T> From<usize> for Id<T> {
    fn from(index: usize) -> Self {
        Self::new(index)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SymbolTag;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TypeTag;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScopeTag;

pub type SymbolId = Id<SymbolTag>;
pub type TypeId = Id<TypeTag>;
pub type ScopeId = Id<ScopeTag>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id: SymbolId = Id::new(42);
        assert_eq!(id.index(), 42);
    }

    #[test]
    fn test_id_from_usize() {
        let id: TypeId = 123.into();
        assert_eq!(id.index(), 123);
    }

    #[test]
    fn test_id_display() {
        let id: ScopeId = Id::new(7);
        assert_eq!(format!("{}", id), "7");
    }

    #[test]
    fn test_id_equality() {
        let id1: SymbolId = Id::new(1);
        let id2: SymbolId = Id::new(1);
        let id3: SymbolId = Id::new(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_type_safety() {
        let symbol_id: SymbolId = Id::new(1);
        let type_id: TypeId = Id::new(1);

        // This would not compile due to type mismatch:
        // assert_eq!(symbol_id, type_id);

        // But we can compare their indices
        assert_eq!(symbol_id.index(), type_id.index());
    }

    #[test]
    fn test_id_is_copy() {
        let id1: SymbolId = Id::new(1);
        let id2 = id1; // This should work because Id implements Copy
        assert_eq!(id1, id2); // Both should be valid
    }
}
