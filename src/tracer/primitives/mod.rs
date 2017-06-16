
pub mod sphere;
pub mod bounding_box;
pub mod triangle;
pub mod light;

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
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}

pub trait HasCenter {
    fn get_center(&self) -> Point3<f32>;
}

pub enum Primitive {
    Sphere(sphere::Sphere),
    Triangle(triangle::Triangle)
}

impl HasBoundingBox for Primitive {
    fn get_bounding_box(&self) -> BoundingBox {
        match self {
            &Primitive::Sphere(ref s) => s.get_bounding_box(),
            &Primitive::Triangle(ref t) => t.get_bounding_box()
        }
    }
}

impl HasColor for Primitive {
    fn get_color(&self) -> Color {
        match self {
            &Primitive::Sphere(ref s) => s.get_color(),
            &Primitive::Triangle(ref t) => t.get_color()
        }
    }
}

impl Intersectable for Primitive {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        match self {
            &Primitive::Sphere(ref s) => s.intersect(ray),
            &Primitive::Triangle(ref t) => t.intersect(ray)
        }
    }  
}

impl HasCenter for Primitive {
    fn get_center(&self) -> Point3<f32> {
        match self {
            &Primitive::Sphere(ref s) => s.get_center(),
            &Primitive::Triangle(ref t) => t.get_center()
        }
    }
}

