use bevy::{ecs::DynamicBundle, prelude::*};

pub trait CommandsExt {
    fn with_child(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> &mut Self;

    fn with_a_child<T, F>(&mut self, func: F) -> &mut Self
    where
        F: FnOnce(Entity) -> T,
        T: DynamicBundle + Send + Sync + 'static;

    fn entity_with_bundle<T>(&mut self, func: impl FnMut(Entity) -> T) -> &mut Self
    where
        T: DynamicBundle + Send + Sync + 'static;

    fn entity(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> Entity;
    fn unwrap_entity(&self) -> Entity;
}

impl CommandsExt for Commands {
    fn with_child(&mut self, bundle: impl DynamicBundle + Send + Sync + 'static) -> &mut Self {
        self.with_children(|it| {
            it.spawn(bundle);
        })
    }

    fn with_a_child<T, F>(&mut self, func: F) -> &mut Self
    where
        F: FnOnce(Entity) -> T,
        T: DynamicBundle + Send + Sync + 'static,
    {
        self.with_children(|it| {
            let e = it.spawn(()).current_entity().unwrap();
            it.with_bundle(func(e));
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

    fn unwrap_entity(&self) -> Entity {
        self.current_entity().unwrap()
    }
}
