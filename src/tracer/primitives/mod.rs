
pub mod sphere;
pub mod bounding_box;

use nalgebra::Point3;

pub use tracer::utils::color::Color;
pub use tracer::utils::intersection::Intersection;
pub use tracer::utils::ray::Ray;
pub use tracer::utils::scene::Scene;

pub use tracer::primitives::bounding_box::BoundingBox;

pub trait HasBoundingBox {
    fn get_bounding_box(&self) -> BoundingBox;
}

pub trait HasColor {
    fn get_color(&self) -> Color;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

pub trait HasCenter {
    fn get_center(&self) -> Point3<f32>;
}