
pub use tracer::primitives::{HasBoundingBox, HasColor, Intersectable, HasCenter};
pub use tracer::primitives::bounding_box::BoundingBox;
pub use tracer::utils::ray::Ray;
pub use tracer::utils::color::Color;
pub use tracer::utils::intersection::Intersection;

use nalgebra::{Point3, Vector3};

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
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut t0: f32;
        let mut t1: f32;
        let l: Vector3<f32> = Vector3::new(self.origin.x - ray.origin.x, 
                                           self.origin.y - ray.origin.y,
                                           self.origin.z - ray.origin.z);
        let tca = l.dot(&ray.direction);
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
 
        return Some(Intersection{
            color: Color::new_copy(&self.color),
            time: t0
        }); 
    }
}

impl HasCenter for Sphere {
    fn get_center(&self) -> Point3<f32> {
        return self.origin;
    }
}