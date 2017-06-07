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
use std::f32;
use std::cmp::Ordering;
use std::collections::HashSet;

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

impl Sphere {
    pub fn get_bounding_box(&self) -> BoundingBox {
        let world_origin = &self.object.to_world.transform_point(&Point3::new(0.0, 0.0, 0.0));
        return BoundingBox {
            min: Point3::new(world_origin.x - self.radius, 
                             world_origin.y - self.radius, 
                             world_origin.z - self.radius),
            max: Point3::new(world_origin.x + self.radius, 
                             world_origin.y + self.radius, 
                             world_origin.z + self.radius)
        }
    }
}

impl Plane {
    //TODO implement
    pub fn get_bounding_box(&self) -> BoundingBox {
        return BoundingBox {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0)
        }
    }
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

pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>
}

impl BoundingBox {
    pub fn intersect_bbox(&self, bbox: &BoundingBox) -> bool {
        if bbox.min.x > self.max.x { return false; }
        if bbox.max.x < self.min.x { return false; }

        if bbox.min.y > self.max.y { return false; }
        if bbox.max.y < self.min.y { return false; }

        if bbox.min.z > self.max.z { return false; }
        if bbox.max.z < self.min.z { return false; }

        return true;
    }
}

impl Intersectable for BoundingBox {
    #[allow(unused_variables)]
    fn intersect(&self, ray: &mut Ray) -> bool {

        //check if origin is in bbox
        if ray.origin.x > self.min.x && ray.origin.x < self.max.x &&
           ray.origin.y > self.min.y && ray.origin.y < self.max.y &&
           ray.origin.z > self.min.z && ray.origin.z < self.max.z {
            ray.intersection.time = 0.0;
            return true;
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
            return false;
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
            return false;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        if tzmax < tmax {
            tmax = tzmax;
        }

        let intersect = tmin < t1 && tmax > t0;

        if intersect {
            ray.intersection.time = tmin;    
        }

        return intersect;
        
    }
}

impl Element {
    pub fn color(&self) -> &Color {
        match *self {
            Element::Sphere(ref s) => &s.object.color,
            Element::Plane(ref p) => &p.object.color,
        }
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        match *self {
            Element::Sphere(ref s) => s.get_bounding_box(),
            Element::Plane(ref p) => p.get_bounding_box(),
        }
    }
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub lights: Light,
    pub bvh: BoundingVolumeHierarchy
}

pub enum BVHNode {
    BVHNodeCore(BVHNodeCore),
    BVHNodeLeaf(BVHNodeLeaf)
}

impl BVHNode {
    pub fn get_bounding_box(&self) -> BoundingBox {
        match *self {
            BVHNode::BVHNodeCore(ref s) => BoundingBox {
                min: s.bbox.min,
                max: s.bbox.max
            },
            BVHNode::BVHNodeLeaf(ref p) => p.element.get_bounding_box(),
        }
    }    
}


pub struct BVHNodeCore {
    pub bbox: BoundingBox,
    pub left_child: Box<BVHNode>,
    pub right_child: Box<BVHNode>
}

pub struct BVHNodeLeaf {
    pub element: Arc<Element>
}

pub struct BoundingVolumeHierarchy {
    pub root_node: Box<BVHNode>
}

impl BVHNodeLeaf {
    pub fn new(element: Arc<Element>) -> BVHNodeLeaf {
        return BVHNodeLeaf {
            element: element
        }
    }
}

impl BoundingVolumeHierarchy {
    pub fn new_from_objects(elements: Vec<Arc<Element>>) -> BoundingVolumeHierarchy {

        //create node for each element
        let mut to_combine: HashSet<Box<BVHNode>> = HashSet::new();
        for elem in &elements {
            to_combine.push(
                Box::new(
                    BVHNode::BVHNodeLeaf(
                        BVHNodeLeaf::new(elem.clone())
                    )
                )
            );
        }

        // 1. order primitive based on bounding box, lower volume at the beginning
        to_combine.sort_by(|a, b| {
            let bbox_a = a.get_bounding_box();
            let bbox_b = b.get_bounding_box();
            let bbox_a_min = bbox_a.min;
            let bbox_a_max = bbox_a.max;
            let bbox_b_min = bbox_b.min;
            let bbox_b_max = bbox_b.max;

            let volume_a = (bbox_a_max.x - bbox_a_min.x).abs() *
                           (bbox_a_max.y - bbox_a_min.y).abs() *
                           (bbox_a_max.z - bbox_a_min.z).abs();
            let volume_b = (bbox_b_max.x - bbox_b_min.x).abs() *
                           (bbox_b_max.y - bbox_b_min.y).abs() *
                           (bbox_b_max.z - bbox_b_min.z).abs();
            if volume_a > volume_b {
                return Ordering::Greater;    
            }
            else {
                return Ordering::Less;
            }
            
        });
        // 2. try to combine elements

        //todo implement non-recursive algorithm
        while to_combine.len() > 1 {


            let new_node: BVHNode = BVHNode::BVHNodeCore(
                BVHNodeCore{
                    bbox: BoundingBox {
                        min: Point3::new(min_x, min_y, min_z),
                        max: Point3::new(max_x, max_y, max_z)
                    },
                    left_child: left,
                    right_child: right
                }
            );

            to_combine.push(Box::new(new_node));
        }

        
        return BoundingVolumeHierarchy {
            root_node: to_combine.remove(0)  
        }
    }
}

pub trait Centroid {
    fn get_center(&self) -> Point3<f32>;
}

pub trait Intersectable {
    fn intersect(&self, ray: &mut Ray) -> bool;
}

impl Intersectable for BoundingVolumeHierarchy {
    fn intersect(&self, ray: &mut Ray) -> bool {
        return self.root_node.intersect(ray);
    }
}

impl Intersectable for BVHNode {
    #[allow(unused_variables)]
    #[allow(unused_mut)]
    fn intersect(&self, ray: &mut Ray) -> bool {
        // if we are a leaf, do actual intersection test, else go down the tree
        match *self {
            BVHNode::BVHNodeCore(ref s) => {

                //do we intersect with our bbox? if so, send ray to child

                let mut tmp_ray = Ray::new_from(ray);

                // println!("{}", s.bbox.min)/;
                // println!("{}", s.bbox.max);
                if !s.bbox.intersect(&mut tmp_ray) {
                    return false;
                }

                let mut tmp_ray_left = Ray::new_from(ray);
                let mut tmp_ray_right = Ray::new_from(ray);

                let mut success_left = s.left_child.get_bounding_box().intersect(&mut tmp_ray_left);
                let mut success_right = s.right_child.get_bounding_box().intersect(&mut tmp_ray_right);

                if !success_left && !success_right {
                    return false;
                }

                if success_left && !success_right {
                    return s.left_child.intersect(ray);
                }

                if success_right && !success_left {
                    return s.right_child.intersect(ray);
                }

                let mut success = false;
                //Both bbox intersected by ray, check which one was intersected first
                if tmp_ray_left.intersection.time < tmp_ray_right.intersection.time {
                    success |= s.left_child.intersect(ray);
                    success |= s.right_child.intersect(ray);
                }
                else {
                    success |= s.right_child.intersect(ray);
                    success |= s.left_child.intersect(ray);
                }

                return success;
            },
            BVHNode::BVHNodeLeaf(ref p) => {
                return p.element.intersect(ray);
            }
        }
    }
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

impl Ray {
    pub fn new_from(ray: &Ray) -> Ray {
        return  Ray {
            origin: ray.origin,
            direction: ray.direction,
            intersection: Intersection::new_empty()
        };
    }
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

impl Centroid for Sphere {
    fn get_center(&self) -> Point3<f32> {
        return  self.object.to_world.transform_point(&Point3::new(0.0, 0.0, 0.0));
    }
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

impl Centroid for Plane {
    fn get_center(&self) -> Point3<f32> {
        return  self.object.to_world.transform_point(
            &Point3::new(self.x_max - self.x_min, 0.0, self.z_max - self.z_min));
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

impl Centroid for Element {
    fn get_center(&self) -> Point3<f32> {
        match *self {
            Element::Sphere(ref s) => s.get_center(),
            Element::Plane(ref p) => p.get_center()
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

        &scene.bvh.intersect(&mut r);

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
        }
    }

    for _ in 0..w {
        for _ in 0..h {
            rx.recv().unwrap();
            pb.inc();
        }
    }

    pb.finish_print("done");

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
        elems.push( Arc::new(gen_random_sphere()));
        // elems.push(
        //     Arc::new(
        //         Element::Sphere(
        //             Sphere {
        //                 radius: 500.0,
        //                 object: Object {
        //                     color: Color {
        //                         red: 1.0,
        //                         green: 0.0,
        //                         blue: 0.0
        //                     },
        //                     to_object: Matrix4::new_translation(&Vector3::new(300.0, 200.0, -100.0)),
        //                     to_world: Matrix4::new_translation(&Vector3::new(-300.0, -200.0, 100.0))
        //                 }
        //             }
        //         )
        //     )
        // );
    }

    // Build acceleration structure
    println!("Building acceleration structure");
    let bvh = BoundingVolumeHierarchy::new_from_objects(elems);

    let scene = Scene {
        width: 1920,
        height: 1080,
        lights: Light {
            position: Point3::new(0.0, 0.0, -100.0)

        },
        bvh: bvh
    };

    println!("Rendering...");
    render(scene);
    
}


