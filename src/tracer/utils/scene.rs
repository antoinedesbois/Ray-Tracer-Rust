
use tracer::primitives::Primitive;
use tracer::primitives::light::Light;

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub primitives: Vec<Primitive>,
    pub light: Light
}