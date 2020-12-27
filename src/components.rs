use bevy::{prelude::*, utils::HashSet};

#[derive(Copy, Clone, Debug)]
pub enum Dress {
    Bitpack(u32, Color),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Marker {
    Wall,
    Chair,
    Table,
    Window,
    Door,
    Bookshelf,
    Mirror,
    Oven,
    Bed,
    Dirt,
    RandomTree,
    PlayerSpawn,
    Player,
    Torch,
}

pub type ProximitySet = HashSet<Entity>;
