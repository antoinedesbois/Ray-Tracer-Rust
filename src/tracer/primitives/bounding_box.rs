
use tracer::primitives::Intersectable;
use tracer::utils::ray::Ray;

use nalgebra::{Point3};

use std::f32;

pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>
}

impl Intersectable for BoundingBox {
    #[allow(unused_variables)]
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
            // return Some(Intersection {
            //     color: Color::new_black(),
            //     time: tmin
            // });
            //TODO
            return None;
        }

        return None;
        
    }
}