
use nalgebra::{Point3, Vector3};
use nalgebra::core::Unit;

pub struct Camera {
   pub u: Unit<Vector3<f32>>,
   pub v: Unit<Vector3<f32>>,
   pub w: Unit<Vector3<f32>>,

   pub eye: Point3<f32>,
   pub look_at: Point3<f32>,
   pub up_vector: Unit<Vector3<f32>>,
   pub distance: f32
}

impl Camera {
   pub fn new(eye: Point3<f32>, 
              look_at: Point3<f32>, 
              up_vector: Vector3<f32>, 
              distance: f32) -> Camera {

      let w = Unit::new_normalize(eye - look_at);
      let o = Unit::new_normalize(up_vector);
      let u = Unit::new_normalize(o.cross(w.as_ref()));
      return Camera {
         u: u,
         v: Unit::new_normalize(u.cross(w.as_ref())),
         w: w,

         eye: eye,
         look_at: look_at,
         up_vector: Unit::new_normalize(up_vector),
         distance: distance
      }
   }
}