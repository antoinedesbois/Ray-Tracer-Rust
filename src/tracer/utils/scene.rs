
use tracer::primitives::light::Light;
use tracer::utils::camera::Camera;
use tracer::utils::bounding_volume_hierarchy::BoundingVolumeHierarchy;

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub light: Light,
    pub camera: Camera,
    pub bvh: BoundingVolumeHierarchy
}