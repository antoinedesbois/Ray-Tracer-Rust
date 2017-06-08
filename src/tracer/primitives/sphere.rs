
use nalgebra::Point3;
use tracer::utils::color::Color;

pub struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
    pub color: Color
}

impl BoundingBox for Sphere {
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

impl Color for Sphere {
    fn get_color(&self) -> Color {
        return self.color;
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        return 
    }
}