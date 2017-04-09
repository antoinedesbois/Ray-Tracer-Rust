extern crate image;
use image::{DynamicImage, Rgba, GenericImage, Pixel};

// use std::time::Duration;
// use std::thread;
use std::fs::File;
use std::path::Path;

pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

pub struct Color {
    pub red : f32,
    pub green : f32,
    pub blue : f32
}

pub struct Sphere {
    pub radius : f32,
    pub center : Point,
    pub color : Color
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub sphere: Sphere
}

pub struct Ray {
    pub origin: Point,
    pub direction: Vector3,
}

const GAMMA: f32 = 2.2;
fn gamma_encode(linear: f32) -> f32 {
    linear.powf(1.0 / GAMMA)
}

impl Color {
    pub fn to_rgba(&self) -> Rgba<u8> {
        Rgba::from_channels((gamma_encode(self.red) * 255.0) as u8,
                            (gamma_encode(self.green) * 255.0) as u8,
                            (gamma_encode(self.blue) * 255.0) as u8,
                            255)
    }
}

fn quadratic_solution(a: f32, b: f32, c: f32) -> (f32, f32) {
    return (((- b + (b*b - 4.0*a*c).sqrt() ) / (2.0 * a)),
            ((- b - (b*b - 4.0*a*c).sqrt() ) / (2.0 * a)));
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> bool;
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> bool {
        let a = ray.direction.x * ray.direction.x +
                ray.direction.y * ray.direction.y +
                ray.direction.z * ray.direction.z;
        let b = (ray.direction.x * ray.origin.x +
                ray.direction.y * ray.origin.y +
                ray.direction.z * ray.origin.z) * 2.0;
        let c = ray.origin.x * ray.origin.x +
                ray.origin.y * ray.origin.y +
                ray.origin.z * ray.origin.z -
                (self.radius * self.radius);

        let (t1, t2) = quadratic_solution(a, b, c);
        return t1 > 0.0 || t2 > 0.0;
    }
}

pub fn render(scene: &Scene) -> DynamicImage {
    let mut img = DynamicImage::new_rgb8(scene.width, scene.height);
    let black = Rgba::from_channels(0, 0, 0, 0);

    for px in 0..scene.width {
        for py in 0..scene.height {
            //Create ray and trace
            let r = Ray {
                origin: Point {
                    x: px as f32,
                    y: py as f32,
                    z: 0.0
                },
                direction: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0
                }
            };

            if scene.sphere.intersect(&r) {
                img.put_pixel(px, py, Rgba::to_rgba(&scene.sphere.color.to_rgba()));
                println!("touch!");
            } else {
                img.put_pixel(px, py, black);
            }
        }
    }

    return img;
}

#[test]
fn test_can_render_scene() {
    use image::GenericImage;
    let scene = Scene {
        width: 800,
        height: 600,
        fov: 90.0,
        sphere: Sphere {
            center: Point {
                x: 400.0,
                y: 300.0,
                z: -5.0,
            },
            radius: 100.0,
            color: Color {
                red: 0.4,
                green: 1.0,
                blue: 0.4,
            },
        },
    };

    let img: DynamicImage = render(&scene);
    assert_eq!(scene.width, img.width());
    assert_eq!(scene.height, img.height());
}

fn main() {
    println!("Ray tracer starting...");

    println!("Building scene");
    let scene = Scene {
        width: 800,
        height: 600,
        fov: 90.0,
        sphere: Sphere {
            center: Point {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            radius: 1.0,
            color: Color {
                red: 0.4,
                green: 1.0,
                blue: 0.4,
            },
        },
    };

    // thread::sleep(Duration::from_millis(100000));

    let img = render(&scene);
    let ref mut fout = File::create(&Path::new("output.png")).unwrap();
    img.save(fout, image::PNG).unwrap();
}
