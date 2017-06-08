
use tracer::utils::color::Color;

use nalgebra::Point3;

use std;

pub struct Intersection {
    pub point: Point3<f32>,
    pub color: Color,
    pub time: f32
}

impl Intersection {
    pub fn new_empty() -> Intersection {
        return Intersection {
           point: Point3::new(0.0, 0.0, 0.0),
            color: Color::new_black(),
            time: std::f32::MAX
        }
    }
}