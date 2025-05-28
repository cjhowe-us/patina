use crate::analysis::id::Id;

#[derive(Debug, Clone)]
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn insert(&mut self, item: T) -> Id<T> {
        let id = Id::new(self.items.len());
        self.items.push(item);
        id
    }

    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.items.get(id.index())
    }

    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        self.items.get_mut(id.index())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub fn iter_with_ids(&self) -> impl Iterator<Item = (Id<T>, &T)> {
        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| (Id::new(index), item))
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for Arena<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct TestItem {
        value: i32,
    }

    #[test]
    fn test_arena_insert_and_get() {
        let mut arena = Arena::new();
        let item1 = TestItem { value: 42 };
        let item2 = TestItem { value: 100 };

        let id1 = arena.insert(item1);
        let id2 = arena.insert(item2);

        assert_eq!(arena.get(id1), Some(&TestItem { value: 42 }));
        assert_eq!(arena.get(id2), Some(&TestItem { value: 100 }));
    }

    #[test]
    fn test_arena_get_mut() {
        let mut arena = Arena::new();
        let item = TestItem { value: 42 };
        let id = arena.insert(item);

        // Use the id by value here, then clone for the second use
        if let Some(item) = arena.get_mut(id) {
            item.value = 100;
        }

        // Since Id implements Copy (due to #[derive(Copy)]), this works
        assert_eq!(arena.get(id), Some(&TestItem { value: 100 }));
    }

    #[test]
    fn test_arena_len_and_is_empty() {
        let mut arena: Arena<TestItem> = Arena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);

        arena.insert(TestItem { value: 1 });
        assert!(!arena.is_empty());
        assert_eq!(arena.len(), 1);

        arena.insert(TestItem { value: 2 });
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_arena_iter() {
        let mut arena = Arena::new();
        arena.insert(TestItem { value: 1 });
        arena.insert(TestItem { value: 2 });
        arena.insert(TestItem { value: 3 });

        let values: Vec<i32> = arena.iter().map(|item| item.value).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_arena_iter_with_ids() {
        let mut arena = Arena::new();
        let id1 = arena.insert(TestItem { value: 10 });
        let id2 = arena.insert(TestItem { value: 20 });

        let items: Vec<(Id<TestItem>, i32)> = arena
            .iter_with_ids()
            .map(|(id, item)| (id, item.value))
            .collect();

        assert_eq!(items[0].0, id1);
        assert_eq!(items[0].1, 10);
        assert_eq!(items[1].0, id2);
        assert_eq!(items[1].1, 20);
    }

    #[test]
    fn test_arena_into_iter() {
        let mut arena = Arena::new();
        arena.insert(TestItem { value: 1 });
        arena.insert(TestItem { value: 2 });

        let values: Vec<i32> = arena.into_iter().map(|item| item.value).collect();
        assert_eq!(values, vec![1, 2]);
    }
}