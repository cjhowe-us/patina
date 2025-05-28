pub mod components;
pub mod builder;
pub mod query;
pub mod converter;

#[cfg(test)]
mod example;

use hecs::{Entity, World};

pub struct AstWorld {
    pub world: World,
    pub root: Option<Entity>,
}

impl AstWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            root: None,
        }
    }

    pub fn set_root(&mut self, entity: Entity) {
        self.root = Some(entity);
    }
}