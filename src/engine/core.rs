use specs::prelude::*;
use raylib::prelude::*;

#[derive(Default)]
pub struct GameStatus {
    pub should_close : bool,
}

pub struct TransformComponent {
    transform: Transform,
}

