
pub use tracer::primitives::{HasBoundingBox, HasColor, Intersectable, HasCenter};
pub use tracer::primitives::bounding_box::BoundingBox;
pub use tracer::utils::ray::Ray;
pub use tracer::utils::color::Color;
pub use tracer::utils::intersection::Intersection;

use nalgebra::{Point3, Vector3, distance};
use nalgebra::core::Unit;

use std::mem;

pub struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
    pub color: Color,

    radius2: f32
}

impl Sphere {
    pub fn new(radius: f32, origin: Point3<f32>, color: Color) -> Sphere {
        return Sphere {
            radius: radius,
            origin: origin,
            color: color,
            radius2: radius*radius
        };
    }
}

impl HasBoundingBox for Sphere {
    fn get_bounding_box(&self) -> BoundingBox {
        return BoundingBox {
            min: Point3::new(self.origin.x - self.radius, 
                             self.origin.y - self.radius, 
                             self.origin.z - self.radius),
            max: Point3::new(self.origin.x + self.radius, 
                             self.origin.y + self.radius, 
                             self.origin.z + self.radius)
        }
    }
}

impl HasColor for Sphere {
    fn get_color(&self) -> Color {
        return Color::new_copy(&self.color);
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let mut t0: f32;
        let mut t1: f32;
        let l: Vector3<f32> = self.origin - ray.origin;
        let tca = l.dot(&ray.direction);
        if tca < 0.0 { //intersection is behind ray origin
            return None;
        }
        let d2: f32 = l.dot(&l) - tca * tca;
        if d2 > self.radius2 {
            return None;
        }

        let thc: f32 = (self.radius2 - d2).sqrt(); 

        t0 = tca - thc; 
        t1 = tca + thc; 

        if t0 > t1 {
            mem::swap(&mut t0, &mut t1);
        }

 
        if t0 < 0.0 { 
            t0 = t1;
            if t0 < 0.0 {
                return None;
            }
        } 
 
        let p_hit = Point3::new(t0 * ray.direction.x, 
                                t0 * ray.direction.y,
                                t0 * ray.direction.z);
        return Some(distance(&p_hit, &ray.origin));
        // return Some(Intersection{
            // color: Color::new_copy(&self.color),
            // time: t0,
            // normal: Unit::new_normalize((ray.origin + t0 * ray.direction.as_ref()) - self.origin)
        // }); 
    }
}

impl HasCenter for Sphere {
    fn get_center(&self) -> Point3<f32> {
        return self.origin;
    }
}