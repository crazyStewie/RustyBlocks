use specs::prelude::*;
use raylib::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct RaylibData;

pub struct RenderSystem{
    pub thread : RaylibThread,
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (WriteExpect<'a, RaylibHandle>, Write<'a, crate::engine::core::GameStatus>);

    fn run(&mut self, (mut handle,mut game_data): Self::SystemData) {
        if handle.window_should_close() {
            game_data.should_close = true;
        };
        let mut draw_handle = handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::WHITE);
        draw_handle.draw_fps(10, 10);
    }

}

pub fn setup_render_system(world: &mut World) -> RaylibThread {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .vsync()
        .build();
    world.insert(rl);
    return thread;
}