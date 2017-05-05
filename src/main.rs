extern crate image;
extern crate nalgebra;
extern crate alga;
extern crate rand;
extern crate num_cpus;
extern crate threadpool;
extern crate pbr;

use image::{DynamicImage, Rgba, GenericImage, Pixel};
use nalgebra::{Point3, Vector3, Matrix4};
use alga::linear::Transformation;
use threadpool::ThreadPool;
use pbr::ProgressBar;

use rand::distributions::{IndependentSample, Range};
use std::sync::{Arc, Mutex, mpsc};
use std::fmt;
use std::fs::File;
use std::path::Path;

pub struct Color {
    pub red : f32,
    pub green : f32,
    pub blue : f32
}

impl Color {
    pub fn new_black() -> Color {
        return Color { red: 0.0, green: 0.0, blue: 0.0 };
    }

    pub fn new_copy(color : &Color) -> Color {
        return Color { red: color.red, green: color.green, blue: color.blue };
    }
}

pub struct Object {
    pub color: Color,
    pub to_object: Matrix4<f32>,
    pub to_world: Matrix4<f32>
}

pub struct Sphere {
    pub radius: f32,
    pub object: Object
}

pub struct Plane {
    pub x_min: f32,
    pub x_max: f32,
    pub z_min: f32,
    pub z_max: f32,
    pub object: Object
}

pub enum Element {
    Sphere(Sphere),
    Plane(Plane)
}

pub struct Light {
    pub position: Point3<f32>
}

impl Element {
    pub fn color(&self) -> &Color {
        match *self {
            Element::Sphere(ref s) => &s.object.color,
            Element::Plane(ref p) => &p.object.color,
        }
    }
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub elements: Vec<Element>,
    pub lights: Light
}

pub struct Intersection {
    pub point: Point3<f32>,
    pub color: Color,
    pub time: f32
}

impl Intersection {
    pub fn new_empty() -> Intersection {
        return Intersection {
           point: Point3::new(0.0, 0.0, 0.0),
            color: Color::new_black(),
            time: std::f32::MAX
        }
    }
}

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub intersection: Intersection
}

const GAMMA: f32 = 2.2;
const NB_RAY: u32 = 100; //Per pixel
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

fn gen_random_sample(lower_bound: f32, upper_bound: f32) -> f32 {
    let mut rng = rand::thread_rng();
    let between = Range::new(lower_bound, upper_bound);

    return between.ind_sample(&mut rng);
}

fn gen_random_color() -> Color {
    return Color { 
        red: gen_random_sample(0.0, 1.0), 
        green: gen_random_sample(0.0, 1.0), 
        blue: gen_random_sample(0.0 ,1.0) 
    }
}

fn gen_random_sphere() -> Element {
    let trans_x = gen_random_sample(-1000.0, 1000.0);
    let trans_y = gen_random_sample(-1000.0, 1000.0);

    return Element::Sphere(
        Sphere {
            radius: gen_random_sample(10.0, 100.0),
            object: Object {
                color: gen_random_color(),
                to_object: Matrix4::new_translation(&Vector3::new(trans_x, trans_y, 1000.0)),
                to_world: Matrix4::new_translation(&Vector3::new(-trans_x, -trans_y, -1000.0))
            }
        }
    );
}
            

fn quadratic_solution(a: f32, b: f32, c: f32) -> (f32, f32) {
    return (((- b + (b*b - 4.0*a*c).sqrt() ) / (2.0 * a)),
            ((- b - (b*b - 4.0*a*c).sqrt() ) / (2.0 * a)));
}

fn get_intersection_point(ray: &Ray, time: f32) -> Point3<f32> {

    return Point3::new(ray.origin.x + time * ray.direction.x,
                       ray.origin.y + time * ray.direction.y,
                       ray.origin.z + time * ray.direction.z);
}

pub trait Transformable {
    fn transform_to(&self, transform: &Matrix4<f32>) -> Self;
}

impl Transformable for Ray {
    fn transform_to(&self, transform: &Matrix4<f32>) -> Self {
        let obj_ray = Ray {
            origin: transform.transform_point(&self.origin),
            direction: transform.transform_vector(&self.direction),
            intersection: Intersection {
                point: self.intersection.point,
                color: Color::new_copy(&self.intersection.color),
                time: self.intersection.time
            }
        };

        return obj_ray;
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "or: ({}, {}, {})",
               self.origin.x,
               self.origin.y,
               self.origin.z)
    }
}

pub trait Intersectable {
    fn intersect(&self, ray: &mut Ray) -> bool;
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &mut Ray) -> bool {

        //bring ray to object space
        let obj_ray = ray.transform_to(&self.object.to_object);

        let a = obj_ray.direction.x * obj_ray.direction.x +
                obj_ray.direction.y * obj_ray.direction.y +
                obj_ray.direction.z * obj_ray.direction.z;
        let b = (obj_ray.direction.x * obj_ray.origin.x +
                obj_ray.direction.y * obj_ray.origin.y +
                obj_ray.direction.z * obj_ray.origin.z) * 2.0;
        let c = obj_ray.origin.x * obj_ray.origin.x +
                obj_ray.origin.y * obj_ray.origin.y +
                obj_ray.origin.z * obj_ray.origin.z -
                (self.radius * self.radius);

        let (t1, t2) = quadratic_solution(a, b, c);

        let mut ray_hit = false;

        if t1 > 0.0 && t1 < ray.intersection.time 
        {
            ray_hit = true;
            ray.intersection.point =
                self.object.to_world.transform_point(&get_intersection_point(&obj_ray, t1));
            ray.intersection.color = Color::new_copy(&self.object.color);
            ray.intersection.time = t1;
        }
        if t2 > 0.0 && t2 < ray.intersection.time
        {
            ray_hit = true;
            ray.intersection.point =
                self.object.to_world.transform_point(&get_intersection_point(&obj_ray, t2));
            ray.intersection.color = Color::new_copy(&self.object.color);
            ray.intersection.time = t2;
        }

        return ray_hit;
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &mut Ray) -> bool {
        let obj_ray = ray.transform_to(&self.object.to_object);

        // first, check against unbounded plane
        let t0: f32 = - obj_ray.origin.y / obj_ray.direction.y;

        if t0 < 0.0 {
            return false;
        }

        let inter_point: Point3<f32> =
            self.object.to_world.transform_point(&get_intersection_point(&obj_ray, t0));

        let point_x: f32 = inter_point.x;
        let point_z: f32 = inter_point.z;
        if point_x > self.x_min && point_x < self.x_max &&
           point_z > self.z_min && point_z < self.z_max && t0 < ray.intersection.time 
        {
            ray.intersection.point = inter_point;
            ray.intersection.color = Color::new_copy(&self.object.color);
            ray.intersection.time = t0;
    		return true;
        }

        return false;
    }
}


impl Intersectable for Element {
    fn intersect(&self, ray: &mut Ray) -> bool {
        match *self {
            Element::Sphere(ref s) => s.intersect(ray),
            Element::Plane(ref p) => p.intersect(ray)
        }
    }
}

pub fn render_pixel(px: u32, py: u32, scene: std::sync::Arc<Scene>) -> Color {
    //Create ray and trace
    let o_x: f32 = px as f32;
    let o_y: f32 = py as f32;
    let trans = Vector3::new(0.0, 0.0, -1.0);
    let w: f32 = scene.width as f32;
    let h: f32 = scene.height as f32;
    let mut avg_col = Color::new_black();

    for _ in 0..NB_RAY {

        let mut r = Ray {
            origin: Point3::new(o_x - w/2.0 + gen_random_sample(0.0, 1.0), 
                                o_y - h/2.0 + gen_random_sample(0.0, 1.0), 
                                0.0),
            direction: trans,
            intersection: Intersection::new_empty()
        };

        for i in &scene.elements {
            i.intersect(&mut r);
        }

        // trace light

        avg_col.red = avg_col.red + (r.intersection.color.red / NB_RAY as f32);
        avg_col.green = avg_col.green + (r.intersection.color.green / NB_RAY as f32);
        avg_col.blue = avg_col.blue + (r.intersection.color.blue / NB_RAY as f32);

    }

    return avg_col;
}

pub fn render(scene: Scene) {
    let w = scene.width;
    let h = scene.height;

    let img = Arc::new(Mutex::new(DynamicImage::new_rgb8(w, h)));
    let scene_ptr = Arc::new(scene);

    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = mpsc::channel();
    let mut pb = ProgressBar::new( (w * h) as u64);
    pb.format("╢▌▌░╟");

    for px in 0..w {
        for py in 0..h {
            // put_pixel use top left pixel as (0, 0)
            let tx = tx.clone();
            let cur_scene = scene_ptr.clone();
            let cur_img = img.clone();

            pool.execute(move || {
                let color = render_pixel(px, py, cur_scene);
                let mut cur_img = cur_img.lock().unwrap();

                cur_img.put_pixel(px, (h -1) - py,
                          Rgba::to_rgba(&color.to_rgba()));

                tx.send(()).unwrap();
            });

            pb.inc();
            
        }
    }

    pb.finish_print("done");

    for _ in 0..w {
        for _ in 0..h {
            rx.recv().unwrap();
        }
    }

    println!("Writting image to disk");
    let ref mut fout = File::create(&Path::new("output.png")).unwrap();
    let img = img.lock().unwrap();
    img.save(fout, image::PNG).unwrap();

}

fn main() {
    println!("Ray tracer starting...");
    println!("Building scene");
    let mut elems = vec![
        // Element::Sphere(
        //     Sphere {
        //         radius: 500.0,
        //         object: Object {
        //             color: Color {
        //                 red: 1.0,
        //                 green: 0.0,
        //                 blue: 0.0
        //             },
        //             to_object: Matrix4::new_translation(&Vector3::new(300.0, 200.0, -100.0)),
        //             to_world: Matrix4::new_translation(&Vector3::new(-300.0, -200.0, 100.0))
        //         }
        //     }
        // )
        // Element::Plane(
        //     Plane {
        //         x_min: -100.0,
        //         x_max: 100.0,
        //         z_min: -100.0,
        //         z_max: 1000.0,
        //         object: Object {
        //             color: Color {
        //                 red: 0.0,
        //                 green: 1.0,
        //                 blue: 0.0
        //             },
        //             to_object: Matrix4::from_axis_angle(&Vector3::x_axis(), 1.2),
        //             to_world: Matrix4::from_axis_angle(&Vector3::x_axis(), 1.2)
        //         }
        //     }
        // )
    ];

    for _ in 0..1000 {
        elems.push( gen_random_sphere());
    }

    let scene = Scene {
        width: 1920,
        height: 1080,
        elements: elems,
        lights: Light {
            position: Point3::new(0.0, 0.0, -100.0)

        }
    };

    println!("Rendering...");
    render(scene);
    
}


