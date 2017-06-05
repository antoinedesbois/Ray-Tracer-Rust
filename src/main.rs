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

#[derive(PartialEq, Eq)]
pub enum Axis {
    AxisX,
    AxisY,
    AxisZ
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
    pub elements: Vec<Arc<Element>>,
    pub lights: Light,
    pub bvh: BoundingVolumeHierarchy
}

pub struct BVHNode {
    pub bbox: BoundingBox,
    pub elements: Vec<Arc<Element>>,
    pub left_child: Option<Arc<BVHNode>>,
    pub right_child: Option<Arc<BVHNode>>
}

pub struct BoundingVolumeHierarchy {
    pub root_node: BVHNode
}



#[allow(unused_variables)]
impl BVHNode {
    pub fn new(elements: Vec<Arc<Element>>) -> BVHNode {
        let mut x_min = f32::MAX;
        let mut y_min = f32::MAX;
        let mut z_min = f32::MAX;

        let mut x_max = f32::MIN;
        let mut y_max = f32::MIN;
        let mut z_max = f32::MIN;

        //Create bounding box for the node
        for elem in &elements {
            let bbox = elem.get_bounding_box();

            if bbox.min.x < x_min {
                x_min = bbox.min.x;
            }
            if bbox.min.y < y_min {
                y_min = bbox.min.y;
            }
            if bbox.min.z < z_min {
                z_min = bbox.min.z;
            }

            if bbox.max.x > x_max {
                x_max = bbox.max.x;
            }
            if bbox.max.y > y_max {
                y_max = bbox.max.y;
            }
            if bbox.max.z > z_max {
                z_max = bbox.max.z;
            }
        }

        let left: Option<Arc<BVHNode>>;
        let right: Option<Arc<BVHNode>>;

        let num_elem = elements.len();
        if num_elem > NB_ELEM_PER_LEAF {

            //Check biggest axis
            let x_delta: f32 = x_max - x_min;
            let y_delta: f32 = y_max - y_min;
            let z_delta: f32 = z_max - z_min;
            let mut v: Vec<Arc<Element>> = elements.clone();

            //construct 2 non-overlapping BoundingBox
            let mut b1 = BoundingBox {
                min: Point3::new(x_min, y_min, z_min),
                max: Point3::new(x_max, y_max, z_max)
            };
            let mut b2 = BoundingBox {
                min: Point3::new(x_min, y_min, z_min),
                max: Point3::new(x_max, y_max, z_max)
            };
            let mut v1: Vec<Arc<Element>> = vec![];
            let mut v2: Vec<Arc<Element>> = vec![];

            if x_delta >= y_delta && x_delta >= z_delta {
                //order on X
                v.sort_by(|a, b| {
                    if a.get_bounding_box().min.x < b.get_bounding_box().min.x {
                        return Ordering::Greater;    
                    }
                    else {
                        return Ordering::Less;
                    }
                    
                });

                b1.min.x = x_min;
                b1.min.y = y_min;
                b1.min.z = z_min;
                b1.max.x = elements[num_elem/2].get_bounding_box().min.x;
                b1.max.y = y_max;
                b1.max.z = z_max;

                
                b2.min.x = b1.max.x;
                b2.min.y = y_min;
                b2.min.z = z_min;
                b2.max.x = elements.last().unwrap().get_bounding_box().max.x;
                b2.max.y = y_max;
                b2.max.z = z_max;

                //push object in appropriate vectors
                for elem in &elements {
                    if elem.get_bounding_box().intersect_bbox(&b1) {
                        v1.push(elem.clone());
                    }
                    if elem.get_bounding_box().intersect_bbox(&b2){
                        v2.push(elem.clone());
                    }
                }
            }
            else if y_delta >= x_delta && y_delta >= z_delta {
                //order on Y
                v.sort_by(|a, b| {
                    if a.get_bounding_box().min.y < b.get_bounding_box().min.y {
                        return Ordering::Greater;    
                    }
                    else {
                        return Ordering::Less;
                    }
                    
                });

                b1.min.x = x_min;
                b1.min.y = y_min;
                b1.min.z = z_min;
                b1.max.x = x_max;
                b1.max.y = elements[num_elem/2].get_bounding_box().min.y;
                b1.max.z = z_max;

                
                b2.min.x = b1.max.x;
                b2.min.y = y_min;
                b2.min.z = z_min;
                b2.max.x = x_max;
                b2.max.y = elements.last().unwrap().get_bounding_box().max.y;
                b2.max.z = z_max;

                //push object in appropriate vectors
                for elem in &elements {
                    if elem.get_bounding_box().intersect_bbox(&b1) {
                        v1.push(elem.clone());
                    }
                    if elem.get_bounding_box().intersect_bbox(&b2){
                        v2.push(elem.clone());
                    }
                }
            }
            else {
                //order on Z
                v.sort_by(|a, b| {
                    if a.get_bounding_box().min.z < b.get_bounding_box().min.z {
                        return Ordering::Greater;    
                    }
                    else {
                        return Ordering::Less;
                    }
                    
                });

                b1.min.x = x_min;
                b1.min.y = y_min;
                b1.min.z = z_min;
                b1.max.x = x_max;
                b1.max.y = y_max;
                b1.max.z = elements[num_elem/2].get_bounding_box().min.z;

                
                b2.min.x = b1.max.x;
                b2.min.y = y_min;
                b2.min.z = z_min;
                b2.max.x = x_max;
                b2.max.y = y_max;
                b2.max.z = elements.last().unwrap().get_bounding_box().max.z;

                //push object in appropriate vectors
                for elem in &elements {
                    if elem.get_bounding_box().intersect_bbox(&b1) {
                        v1.push(elem.clone());
                    }
                    if elem.get_bounding_box().intersect_bbox(&b2){
                        v2.push(elem.clone());
                    }
                }
            }


            let len_dif: i32 = (v1.len() - v2.len()) as i32;
            if len_dif.abs() > 100 {
                left = Some(Arc::new(BVHNode::new_from_bbox(v1, b1)));
                right = Some(Arc::new(BVHNode::new_from_bbox(v2, b2)));

            }
            else {
                left = None;
                right = None;
            }
        }
        else {
            left = None;
            right = None;
        }

        return BVHNode {
            bbox: BoundingBox {
                min: Point3::new(x_min, y_min, z_min),
                max: Point3::new(x_max, y_max, z_max)
            },
            elements: elements,
            left_child: left,
            right_child: right
        }
    }

    pub fn new_from_bbox(elements: Vec<Arc<Element>>, bbox: BoundingBox) -> BVHNode {
        let mut x_min = f32::MAX;
        let mut y_min = f32::MAX;
        let mut z_min = f32::MAX;

        let mut x_max = f32::MIN;
        let mut y_max = f32::MIN;
        let mut z_max = f32::MIN;

        //Create bounding box for the node
        for elem in &elements {
            let bbox = elem.get_bounding_box();

            if bbox.min.x < x_min {
                x_min = bbox.min.x;
            }
            if bbox.min.y < y_min {
                y_min = bbox.min.y;
            }
            if bbox.min.z < z_min {
                z_min = bbox.min.z;
            }

            if bbox.max.x > x_max {
                x_max = bbox.max.x;
            }
            if bbox.max.y > y_max {
                y_max = bbox.max.y;
            }
            if bbox.max.z > z_max {
                z_max = bbox.max.z;
            }
        }

        let left: Option<Arc<BVHNode>>;
        let right: Option<Arc<BVHNode>>;

        let num_elem = elements.len();
        if num_elem > NB_ELEM_PER_LEAF {

            //Check biggest axis
            let x_delta: f32 = bbox.max.x - bbox.min.x;
            let y_delta: f32 = bbox.max.y - bbox.min.y;
            let z_delta: f32 = bbox.max.z - bbox.min.z;
            let mut v: Vec<Arc<Element>> = elements.clone();

            //construct 2 non-overlapping BoundingBox
            let mut b1 = BoundingBox {
                min: Point3::new(x_min, y_min, z_min),
                max: Point3::new(x_max, y_max, z_max)
            };
            let mut b2 = BoundingBox {
                min: Point3::new(x_min, y_min, z_min),
                max: Point3::new(x_max, y_max, z_max)
            };
            let mut v1: Vec<Arc<Element>> = vec![];
            let mut v2: Vec<Arc<Element>> = vec![];

            if x_delta >= y_delta && x_delta >= z_delta {
                //order on X
                v.sort_by(|a, b| {
                    if a.get_bounding_box().min.x < b.get_bounding_box().min.x {
                        return Ordering::Greater;    
                    }
                    else {
                        return Ordering::Less;
                    }
                    
                });

                b1.min.x = x_min;
                b1.min.y = y_min;
                b1.min.z = z_min;
                b1.max.x = elements[num_elem/2].get_bounding_box().min.x;
                b1.max.y = y_max;
                b1.max.z = z_max;

                
                b2.min.x = b1.max.x;
                b2.min.y = y_min;
                b2.min.z = z_min;
                b2.max.x = elements.last().unwrap().get_bounding_box().max.x;
                b2.max.y = y_max;
                b2.max.z = z_max;

                //push object in appropriate vectors
                for elem in &elements {
                    if elem.get_bounding_box().intersect_bbox(&b1) {
                        v1.push(elem.clone());
                    }
                    if elem.get_bounding_box().intersect_bbox(&b2){
                        v2.push(elem.clone());
                    }
                }
            }
            else if y_delta >= x_delta && y_delta >= z_delta {
                //order on Y
                v.sort_by(|a, b| {
                    if a.get_bounding_box().min.y < b.get_bounding_box().min.y {
                        return Ordering::Greater;    
                    }
                    else {
                        return Ordering::Less;
                    }
                    
                });

                b1.min.x = x_min;
                b1.min.y = y_min;
                b1.min.z = z_min;
                b1.max.x = x_max;
                b1.max.y = elements[num_elem/2].get_bounding_box().min.y;
                b1.max.z = z_max;

                
                b2.min.x = b1.max.x;
                b2.min.y = y_min;
                b2.min.z = z_min;
                b2.max.x = x_max;
                b2.max.y = elements.last().unwrap().get_bounding_box().max.y;
                b2.max.z = z_max;

                //push object in appropriate vectors
                for elem in &elements {
                    if elem.get_bounding_box().intersect_bbox(&b1) {
                        v1.push(elem.clone());
                    }
                    if elem.get_bounding_box().intersect_bbox(&b2){
                        v2.push(elem.clone());
                    }
                }
            }
            else {
                //order on Z
                v.sort_by(|a, b| {
                    if a.get_bounding_box().min.z < b.get_bounding_box().min.z {
                        return Ordering::Greater;    
                    }
                    else {
                        return Ordering::Less;
                    }
                    
                });

                b1.min.x = x_min;
                b1.min.y = y_min;
                b1.min.z = z_min;
                b1.max.x = x_max;
                b1.max.y = y_max;
                b1.max.z = elements[num_elem/2].get_bounding_box().min.z;

                
                b2.min.x = b1.max.x;
                b2.min.y = y_min;
                b2.min.z = z_min;
                b2.max.x = x_max;
                b2.max.y = y_max;
                b2.max.z = elements.last().unwrap().get_bounding_box().max.z;

                //push object in appropriate vectors
                for elem in &elements {
                    if elem.get_bounding_box().intersect_bbox(&b1) {
                        v1.push(elem.clone());
                    }
                    if elem.get_bounding_box().intersect_bbox(&b2){
                        v2.push(elem.clone());
                    }
                }
            }

            let len_dif: i32 = (v1.len() - v2.len()) as i32;
            if len_dif.abs() > 100 {
                left = Some(Arc::new(BVHNode::new_from_bbox(v1, b1)));
                right = Some(Arc::new(BVHNode::new_from_bbox(v2, b2)));

            }
            else {
                left = None;
                right = None;
            }
        }
        else {
            left = None;
            right = None;
        }

        return BVHNode {
            bbox: bbox,
            elements: elements,
            left_child: left,
            right_child: right
        }
    }
}

impl BoundingVolumeHierarchy {
    pub fn new_from_objects(elements: Vec<Arc<Element>>) -> BoundingVolumeHierarchy {
        return BoundingVolumeHierarchy {
            root_node: BVHNode::new(elements)
        }
    }
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

        let mut ray_hit = false;
        if self.left_child.is_none() && self.right_child.is_none() {
            for i in &self.elements {
                ray_hit |= i.intersect(ray);
            }
        } else {
            assert!(self.left_child.is_some() && self.right_child.is_some());

            // check which subtree to explore first
            let mut r_left: Ray = Ray::new_from(ray);
            let mut r_right: Ray = Ray::new_from(ray);

            let left_chil_bbox = &self.left_child.clone().unwrap().bbox;
            let left_intersect = left_chil_bbox.intersect(&mut r_left);

            let right_chil_bbox = &self.right_child.clone().unwrap().bbox;
            let right_intersect = right_chil_bbox.intersect(&mut r_right);

            if left_intersect && right_intersect {
                //check which intersected first
                if r_left.intersection.time < r_right.intersection.time {
                    ray_hit |= self.left_child.clone().unwrap().intersect(ray);

                    if !ray_hit {
                        ray_hit |= self.right_child.clone().unwrap().intersect(ray);
                    }
                }
                else {
                    ray_hit |= self.right_child.clone().unwrap().intersect(ray);

                    if !ray_hit {
                        ray_hit |= self.left_child.clone().unwrap().intersect(ray);
                    }
                }
            }
            else if left_intersect {
                ray_hit = self.left_child.clone().unwrap().intersect(ray);
            }
            else if right_intersect {
                ray_hit = self.right_child.clone().unwrap().intersect(ray);
            }
        }


        // let mut ray_hit = false;
        // for i in &self.elements {
        //     ray_hit |= i.intersect(ray);
        // }
        return ray_hit;
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
const NB_RAY: u32 = 10; //Per pixel
const NB_ELEM_PER_LEAF: usize = 50;
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

    for _ in 0..100000 {
        elems.push( Arc::new(gen_random_sphere()));
    }

    let elems2 = vec![];

    // Build acceleration structure
    println!("Building acceleration structure");
    let bvh = BoundingVolumeHierarchy::new_from_objects(elems);

    let scene = Scene {
        width: 1920,
        height: 1080,
        elements: elems2,
        lights: Light {
            position: Point3::new(0.0, 0.0, -100.0)

        },
        bvh: bvh
    };

    println!("Rendering...");
    render(scene);
    
}


