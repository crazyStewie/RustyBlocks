use raylib::prelude::*;
use specs::prelude::*;
use std::sync::{Arc, Mutex};

pub struct ModelComponent {
    pub model : Model,
}
unsafe impl Send for ModelComponent{}
unsafe impl Sync for ModelComponent{}

impl Component for ModelComponent {
    type Storage = VecStorage<Self>;
}


struct CameraComponent {
    camera : Camera,
}
