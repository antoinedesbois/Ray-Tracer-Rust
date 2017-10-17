
use tracer::primitives::Intersectable;
use tracer::primitives::Primitive;
use tracer::primitives::HasBoundingBox;
use tracer::utils::ray::Ray;
use tracer::primitives::HasCenter;

use nalgebra::{Point3};

use std::f32;

// Axis aligned
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>
}

impl BoundingBox
{
    pub fn new(primitive: &Primitive) -> BoundingBox
    {
      return primitive.get_bounding_box();
    }

    pub fn new_from(left: &BoundingBox, right: &BoundingBox) -> BoundingBox
    {
      let min_x: f32 = 
         if left.min.x < right.min.x 
         {
            left.min.x
         }
         else 
         {
            right.min.x
             
         };
      let min_y: f32 = 
         if left.min.y < right.min.y 
         {
            left.min.y
         }
         else 
         {
            right.min.y
             
         };
      let min_z: f32 = 
         if left.min.z < right.min.z
         {
            left.min.z
         }
         else 
         {
            right.min.z
             
         };

      let max_x: f32 = 
         if left.max.x > right.max.x 
         {
            left.max.x
         }
         else 
         {
            right.max.x
             
         };
      let max_y: f32 = 
         if left.max.y > right.max.y 
         {
            left.max.y
         }
         else 
         {
            right.max.y
             
         };
      let max_z: f32 = 
         if left.max.z > right.max.z
         {
            left.max.z
         }
         else 
         {
            right.max.z
             
         };

      let min = Point3::new(min_x, min_y, min_z);
      let max = Point3::new(max_x, max_y, max_z);
      return BoundingBox {
         min: min,
         max: max
      };

    }
}

impl Intersectable for BoundingBox {

    fn intersect(&self, ray: &Ray) -> Option<f32> {

        //check if origin is in bbox
        if ray.origin.x > self.min.x && ray.origin.x < self.max.x &&
           ray.origin.y > self.min.y && ray.origin.y < self.max.y &&
           ray.origin.z > self.min.z && ray.origin.z < self.max.z {
            return Some(0.0);
        }

        let t0 = 0.0;
        let t1 = f32::MAX;

        let mut tmin: f32;
        let mut tmax: f32;
        let tymin: f32;
        let tymax: f32;
        let tzmin: f32;
        let tzmax: f32;

        if ray.direction.x >= 0.0 {
            tmin = (self.min.x - ray.origin.x) / ray.direction.x;
            tmax = (self.max.x - ray.origin.x) / ray.direction.x;
        }
        else {
            tmin = (self.max.x - ray.origin.x) / ray.direction.x;
            tmax = (self.min.x - ray.origin.x) / ray.direction.x;
        }

        if ray.direction.y >= 0.0 {
            tymin = (self.min.y - ray.origin.y) / ray.direction.y;
            tymax = (self.max.y - ray.origin.y) / ray.direction.y;
        }
        else {
            tymin = (self.max.y - ray.origin.y) / ray.direction.y;
            tymax = (self.min.y - ray.origin.y) / ray.direction.y;
        }
        
        if tmin > tymax || tymin > tmax {
            return None;
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        if ray.direction.z >= 0.0 {
            tzmin = (self.min.z - ray.origin.z) / ray.direction.z;
            tzmax = (self.max.z - ray.origin.z) / ray.direction.z;
        }
        else {
            tzmin = (self.max.z - ray.origin.z) / ray.direction.z;
            tzmax = (self.min.z - ray.origin.z) / ray.direction.z;
        }

        if tmin > tzmax || tzmin > tmax {
            return None;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        if tzmax < tmax {
            tmax = tzmax;
        }

        let intersect = tmin < t1 && tmax > t0;

        if intersect {
            assert!(tmin <= tmax);
            return Some(tmin);
        }

        return None;
        
    }
}

impl HasCenter for BoundingBox
{
   fn get_center(&self) -> Point3<f32> {
        return Point3::new(self.max.x - self.min.x,
                           self.max.y - self.min.y,
                           self.max.z - self.min.z);
    }
}