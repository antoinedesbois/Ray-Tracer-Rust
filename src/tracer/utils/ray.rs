
use nalgebra::{Point3, Vector3};
use nalgebra::core::Unit;

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Unit<Vector3<f32>>
}


impl Ray {
   pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Ray {
      return Ray {
         origin: origin,
         direction: Unit::new_normalize(direction)
      }
   }
}