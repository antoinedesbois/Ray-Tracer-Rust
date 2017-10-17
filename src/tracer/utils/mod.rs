
pub mod color;
pub mod intersection;
pub mod scene;
pub mod ray;
pub mod camera;
pub mod bounding_volume_hierarchy;

pub use tracer::utils::ray::Ray;
pub use tracer::utils::color::Color;
pub use tracer::utils::scene::Scene;
pub use tracer::utils::camera::Camera;
pub use tracer::utils::bounding_volume_hierarchy::BoundingVolumeHierarchy;
pub use tracer::utils::bounding_volume_hierarchy::HitInfo;
