pub mod engine;
pub mod base;
use raylib::prelude::*;
use specs::prelude::*;

fn main() {
    let mut world = World::new();
    world.insert(engine::core::GameStatus{should_close:false});
    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(engine::render::render_system::RenderSystem{thread : engine::render::render_system::setup_render_system(&mut world)})
        .build();

    while world.fetch::<engine::core::GameStatus>().should_close == false {
        dispatcher.dispatch(&world);
    }
}
