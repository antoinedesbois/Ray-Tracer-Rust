
use nalgebra::Point3;
use tracer::primitives::Primitive;
use tracer::primitives::CanSample;

pub struct Light {
   pub primitives: Vec<Primitive>
}

impl CanSample for Light {
    fn get_sample(&self, u: f32, v: f32) -> Point3<f32> {
        return self.primitives[0].get_sample(u, v);
    }
}