
use nalgebra::{Point3, Vector3};

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>
}

// impl Ray {
//     pub fn new_from(ray: &Ray) -> Ray {
//         return  Ray {
//             origin: ray.origin,
//             direction: ray.direction
//         };
//     }
// }