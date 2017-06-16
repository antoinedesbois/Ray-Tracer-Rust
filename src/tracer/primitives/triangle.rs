

pub use tracer::primitives::{HasBoundingBox, HasColor, Intersectable, HasCenter};
pub use tracer::primitives::bounding_box::BoundingBox;
pub use tracer::utils::ray::Ray;
pub use tracer::utils::color::Color;
pub use tracer::utils::intersection::Intersection;

use nalgebra::{Point3, Vector3};
use nalgebra::core::Unit;

pub struct Triangle {
    pub v0: Point3<f32>,
    pub v1: Point3<f32>,
    pub v2: Point3<f32>,
    pub color: Color,
    e1: Vector3<f32>,
    e2: Vector3<f32>
}

impl Triangle {
    pub fn new(v0: Point3<f32>, v1: Point3<f32>, v2: Point3<f32>, color: Color) -> Triangle{
        return Triangle {
            v0: v0,
            v1: v1,
            v2: v2,
            color: color,
            e1: v1 - v0,
            e2: v2 - v0
        }
    }
}

fn min_float(v0: f32, v1: f32) -> f32{
    return if v0 < v1 { v0 } else { v1 }
}

fn max_float(v0: f32, v1: f32) -> f32{
    return if v0 < v1 { v1 } else { v0 }
}

impl HasBoundingBox for Triangle {
    fn get_bounding_box(&self) -> BoundingBox {
        return BoundingBox {
            min: Point3::new(min_float(min_float(self.v0.x, self.v1.x), self.v2.x),
                             min_float(min_float(self.v0.y, self.v1.y), self.v2.y),
                             min_float(min_float(self.v0.z, self.v1.z), self.v2.z)),
            max: Point3::new(max_float(max_float(self.v0.x, self.v1.x), self.v2.x),
                             max_float(max_float(self.v0.y, self.v1.y), self.v2.y),
                             max_float(max_float(self.v0.z, self.v1.z), self.v2.z))
        }
   }
}

impl HasColor for Triangle {
    fn get_color(&self) -> Color {
        return Color::new_copy(&self.color);
    }
}

// Möller–Trumbore
impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<f32> {

      // Calculate planes normal vector
      let pvec: Vector3<f32> = ray.direction.cross(&self.e2);
      let det: f32 = self.e1.dot(&pvec);
   
      // Ray is parallel to plane
      if det.abs() < 0.0000001 {
          return None;
      }

      let inv_det: f32 = 1.0 / det;
      let tvec: Vector3<f32> = ray.origin - self.v0;
      let u: f32 = tvec.dot(&pvec) * inv_det;
      if u < 0.0 || u > 1.0 {
          return None;
      }

      let qvec: Vector3<f32> = tvec.cross(&self.e1);
      let v: f32 = ray.direction.dot(&qvec) * inv_det;
      if v < 0.0 || u + v > 1.0 {
          return None;
      }

      // W = 1 - u - v

      let distance = self.e2.dot(&qvec) * inv_det;
      return Some(distance);
    }
}

impl HasCenter for Triangle {
    fn get_center(&self) -> Point3<f32> {
      let bbox: BoundingBox = self.get_bounding_box();
        return Point3::new(bbox.max.x - bbox.min.x,
                           bbox.max.y - bbox.min.y,
                           bbox.max.z - bbox.min.z);
    }
}