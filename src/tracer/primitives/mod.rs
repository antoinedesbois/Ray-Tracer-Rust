
mod sphere;

pub use tracer::primitives::sphere::Sphere;

pub trait HasBoundingBox {
    fn get_bounding_box() -> BoundingBox;
}

pub trait HasColor {
    fn get_color() -> Color;
}

pub trait Intersectable {
    fn intersect() -> Option<Intersection>;
}

pub trait HasCenter {
    fn get_center() -> Point3<f32>;
}