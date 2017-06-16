
use tracer::primitives::Primitive;

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub primitives: Vec<Primitive>
}