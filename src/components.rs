use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum Dress {
    Bitpack(u32, Color),
}
