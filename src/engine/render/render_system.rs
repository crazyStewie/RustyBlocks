use specs::prelude::*;
use raylib::prelude::*;
use std::borrow::BorrowMut;

#[derive(Default)]
struct RaylibData;

pub struct RenderSystem{
    pub thread : RaylibThread,
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (WriteExpect<'a, RaylibHandle>, WriteExpect<'a, Camera3D>, Write<'a, crate::engine::core::GameStatus>, ReadStorage<'a, crate::engine::render::components::ModelComponent>);

    fn run(&mut self, (mut handle,mut camera, mut game_data, models): Self::SystemData) {
        if handle.window_should_close() {
            game_data.should_close = true;
        };
        handle.update_camera(camera.borrow_mut());
        let mut draw_handle = handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::DARKGRAY);
        {
            let mut mode3d = draw_handle.begin_mode_3D(*camera);
            mode3d.draw_grid(10, 1.0);
            mode3d.draw_gizmo(Vector3::zero());
            for model in models.join() {
                mode3d.draw_model(&model.model, Vector3::new(1.0,1.0,1.0), 1.0, Color::WHITE);

            }
        }
        draw_handle.draw_fps(10, 10);

    }

}

pub fn setup_render_system(world: &mut World) -> RaylibThread {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .vsync()
        .resizable()
        .build();
    let mut camera = Camera3D::perspective(Vector3::new(0.0, 5.0, 0.0), Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), 75.0);
    //rl.set_camera_mode(camera, raylib::consts::CameraMode::CAMERA_FREE);
    world.register::<crate::engine::render::components::ModelComponent>();
    world.insert(camera);
    world.insert(rl);

    return thread;
}