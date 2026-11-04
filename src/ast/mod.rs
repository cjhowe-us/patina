//! Component-based AST system for Patina using hecs ECS
//! This module provides an AST structure where:
//! - Child nodes are stored directly in parent structs
//! - Each node has an associated entity ID in the ECS world
//! - Components are stored using generics for type safety
//! - Generic visitors can traverse based on any trait

use hecs::{Entity, World};
use std::collections::HashSet;
use std::marker::PhantomData;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

pub mod components;
pub mod nodes;
pub mod printer;
#[cfg(test)]
pub mod test_utils;
pub mod visitor;

pub use components::*;
pub use nodes::*;
pub use printer::*;
pub use visitor::*;

/// Trait for all AST nodes to get their associated entity ID
pub trait AstNode {
    /// Get the entity ID associated with this AST node
    fn entity_id(&self) -> Entity;
}

/// Generic AST structure with component type parameters
pub struct Ast<C = ()> {
    /// The ECS world containing all entities and components
    pub world: World,
    /// Set of entities with specific component types
    pub component_entities: HashSet<Entity>,
    /// The root node of the AST
    pub root: Root,
    /// Phantom data for component type
    _phantom: PhantomData<C>,
}

impl<C> std::fmt::Debug for Ast<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ast")
            .field("world", &format!("World[{} entities]", self.world.len()))
            .field("component_entities", &self.component_entities)
            .field("root", &self.root)
            .finish()
    }
}

impl Default for Ast<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Ast<()> {
    /// Create a new AST with no specific component types
    pub fn new() -> Self {
        let mut world = World::new();
        let root_entity = world.spawn(());
        Self {
            world,
            component_entities: HashSet::new(),
            root: Root {
                entity: root_entity,
                statements: Vec::new(),
            },
            _phantom: PhantomData,
        }
    }
}

impl<C> Ast<C> {
    /// Create an entity
    pub fn create_entity(&mut self) -> Entity {
        self.world.spawn(())
    }

    /// Add a component to an entity and track it
    pub fn add_component<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) {
        self.world.insert_one(entity, component).unwrap();
        self.component_entities.insert(entity);
    }

    /// Get a component from an entity
    pub fn get_component<T: 'static + Send + Sync>(
        &self,
        entity: Entity,
    ) -> Option<hecs::Ref<'_, T>> {
        self.world.get::<&T>(entity).ok()
    }

    /// Get a mutable component from an entity  
    pub fn get_component_mut<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Option<hecs::RefMut<'_, T>> {
        self.world.get::<&mut T>(entity).ok()
    }

    /// Query entities with specific components
    pub fn query_entities<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    /// Filter entities by component presence
    pub fn filter_entities<T: 'static + Send + Sync>(&self) -> HashSet<Entity> {
        let mut entities = HashSet::new();
        for (entity, _) in self.world.query::<&T>().iter() {
            entities.insert(entity);
        }
        entities
    }
}

/// Generic visitor trait that can work with any component constraint
pub trait GenericVisitor<C> {
    /// Visit any node that satisfies the component constraint
    fn visit_node<N: AstNode>(&mut self, ast: &Ast<C>, node: &N)
    where
        Self: Sized;
}

/// Trait for traversable nodes
pub trait Traversable<C> {
    /// Accept a generic visitor
    fn accept<V: GenericVisitor<C>>(&self, visitor: &mut V, ast: &Ast<C>)
    where
        Self: AstNode + Sized,
    {
        visitor.visit_node(ast, self);
    }
}
