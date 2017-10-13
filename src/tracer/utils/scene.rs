
use tracer::primitives::Primitive;
use tracer::primitives::light::Light;
use tracer::utils::camera::Camera;

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub primitives: Vec<Primitive>,
    pub light: Light,
    pub camera: Camera
}