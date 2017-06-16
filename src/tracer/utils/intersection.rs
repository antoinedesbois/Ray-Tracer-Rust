
use tracer::utils::color::Color;
use nalgebra::Vector3;
use nalgebra::core::Unit;

pub struct Intersection {
    pub color: Color,
    pub time: f32,
    pub normal: Unit<Vector3<f32>>
}