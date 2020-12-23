use bevy::{ecs::DynamicBundle, prelude::*};

pub trait CommandsExt {
    fn with_child(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> &mut Self;

    fn entity_with_bundle<T>(&mut self, func: impl FnMut(Entity) -> T) -> &mut Self
    where
        T: DynamicBundle + Send + Sync + 'static;

    fn entity(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> Entity;
}

impl CommandsExt for Commands {
    fn with_child(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> &mut Self {
        self.with_children(|it| {
            it.spawn(bundle);
        })
    }

    fn entity_with_bundle<T>(&mut self, func: impl FnMut(Entity) -> T) -> &mut Self
    where
        T: DynamicBundle + Send + Sync + 'static,
    {
        if let Some(bundle) = self.current_entity().map(func) {
            self.with_bundle(bundle);
        }
        self
    }

    fn entity(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> Entity {
        self.spawn(bundle).current_entity().unwrap()
    }
}
