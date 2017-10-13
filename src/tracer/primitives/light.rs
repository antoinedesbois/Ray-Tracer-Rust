
use nalgebra::Point3;
use tracer::primitives::Primitive;

pub struct Light {
   pub position: Point3<f32>,
   pub primitives: Vec<Primitive>
}