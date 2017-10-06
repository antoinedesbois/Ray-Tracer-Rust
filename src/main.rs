extern crate image;
extern crate nalgebra;
extern crate alga;
extern crate rand;
extern crate num_cpus;
extern crate threadpool;
extern crate time;

mod tracer;

use tracer::primitives::Primitive;
use tracer::primitives::sphere::Sphere;
use tracer::primitives::triangle::Triangle;
use tracer::primitives::light::Light;

use tracer::primitives::Intersectable;
use tracer::primitives::HasColor;
use tracer::primitives::HasNormal;

use tracer::utils::scene::Scene;
use tracer::utils::color::Color;
use tracer::utils::ray::Ray;
use tracer::utils::camera::Camera;

use image::{DynamicImage, Rgba, GenericImage, Pixel};
use nalgebra::{Point3, Vector3, distance};

use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex, mpsc};
use std::path::Path;
use std::f32;
use std::thread;
use std::env;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

const NB_RAY: u32 = 25; //Per pixel
const NB_RAND_SAMPLE: u32 = 2000000;

#[allow(unused_variables)]
#[allow(dead_code)]
fn gen_random_spheres() -> Vec<Primitive> {

    let mut primitives: Vec<Primitive> = Vec::new();
    let mut rng = rand::thread_rng();
    let between_0_1 = Range::new(0.0, 1.0);
    let between_0_500 = Range::new(400.0, 500.0);
    let between_n500_500 = Range::new(-500.0, 500.0);

    for _ in 0..1 {
        primitives.push(
            Primitive::Sphere(
                Sphere::new(
                        between_0_500.ind_sample(&mut rng), 
                        Point3::new(0.0/*between_n500_500.ind_sample(&mut rng)*/, 
                                    0.0/*between_n500_500.ind_sample(&mut rng)*/, 
                                    -1000.0), 
                        Color::new(between_0_1.ind_sample(&mut rng),
                                   between_0_1.ind_sample(&mut rng),
                                   between_0_1.ind_sample(&mut rng))
                )
            )
        );
    }

    return primitives;
}

#[allow(dead_code)]
fn gen_random_triangles() -> Vec<Primitive> {

    let mut primitives: Vec<Primitive> = Vec::new();
    let mut rng = rand::thread_rng();
    let between_0_1 = Range::new(0.0, 1.0);
    let between_n500_n1000 = Range::new(-100.0, -50.0);
    let between_n500_500 = Range::new(-500.0, 500.0);

    for _ in 0..10 {
        primitives.push(
            Primitive::Triangle(
                Triangle::new(Point3::new(between_n500_500.ind_sample(&mut rng), 
                                          between_n500_500.ind_sample(&mut rng), 
                                          between_n500_n1000.ind_sample(&mut rng)),
                              Point3::new(between_n500_500.ind_sample(&mut rng), 
                                          between_n500_500.ind_sample(&mut rng), 
                                          between_n500_n1000.ind_sample(&mut rng)),
                              Point3::new(between_n500_500.ind_sample(&mut rng),
                                          between_n500_500.ind_sample(&mut rng),
                                          between_n500_n1000.ind_sample(&mut rng)), 
                              Color::new(between_0_1.ind_sample(&mut rng),
                                         between_0_1.ind_sample(&mut rng),
                                         between_0_1.ind_sample(&mut rng))
                )
            )
        );
    }

    return primitives;
}

#[allow(dead_code)]
fn create_ground() -> Vec<Primitive> {
    return vec![Primitive::Triangle(
                Triangle::new(
                    Point3::new(-10000.0, 0.0, -10000.0),
                    Point3::new(10000.0, 0.0, -10000.0),
                    Point3::new(0.0, 0.0, 10000.0),
                    Color::new(0.5, 0.5, 0.5)
                )
            )];
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn import_obj(path: &std::string::String) -> Vec<Primitive> {
    let f = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            println!("Not a valid path: {}", path);
            return vec![];
        }
    };

    let mut primitives: Vec<Primitive> = Vec::new();
    let mut vertices: Vec<Point3<f32>> = Vec::new();
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let tokens: Vec<&str> = l.split(" ").collect();

        if tokens[0] == "v" {
            let x: f32 = tokens[1].parse().unwrap();
            let y: f32 = tokens[2].parse().unwrap();
            let z: f32 = tokens[3].parse().unwrap();

            vertices.push(Point3::new(x, y, z));
        }
        else if tokens[0] == "f" { //We expect face to be triangles
            let idx0: usize = tokens[1].parse().unwrap();
            let idx1: usize = tokens[2].parse().unwrap();
            let idx2: usize = tokens[3].parse().unwrap();
            let t = Primitive::Triangle(
                        Triangle::new(vertices[idx0 - 1], vertices[idx1 - 1], vertices[idx2 - 1],
                                      Color::new(1.0, 1.0, 1.0)));
            primitives.push(t);
        }
    } 

    return primitives;
}

#[allow(unused_variables)]
fn create_rays(px: u32, py: u32, scene: &Scene, random_samples: &Vec<(f32, f32)>) -> Vec<Ray> {
    
    let mut rays: Vec<Ray> = Vec::with_capacity(NB_RAY as usize);
    let o_x: f32 = px as f32;
    let o_y: f32 = py as f32;
    let w: f32 = scene.width as f32;
    let h: f32 = scene.height as f32;

    let width_height_ratio: f32 = w / h;

    for i in  0..NB_RAY {
        let direction = 
                (o_x - w / 2.0 +
                    random_samples[((px * scene.width + py + i) % NB_RAND_SAMPLE) as usize].0) * 
                    scene.camera.u.as_ref() +
                (o_y - h / 2.0 + 
                    random_samples[((px * scene.width + py + i) % NB_RAND_SAMPLE) as usize].1) * 
                    scene.camera.v.as_ref() -
                scene.camera.distance * scene.camera.w.as_ref();

        rays.push(
            Ray::new(
                scene.camera.eye,
                direction
            )
        );
    }

    return rays;
}

pub fn render_pixel(px: u32, py: u32, scene: &Scene, random_samples: &Vec<(f32, f32)>) -> Color {

    let mut avg_col = Color::new_black();

    // Will not trace more rays per pixel than allowed to fit in the vector
    let rays: Vec<Ray> = create_rays(px, py, scene, random_samples);
    for r in rays {

        let mut closest_intersection = f32::MAX;
        let mut hit_primitive = None;

        for el in &scene.primitives {
            let intersection = el.intersect(&r);    

            match intersection {
                Some(x) => {
                    if x >= 0.0 && x < closest_intersection {
                        closest_intersection = x;
                        hit_primitive = Some(el);
                    }
                },
                None    => {},
            }
        }

        // if no intersection, continue
        if closest_intersection == f32::MAX {
            continue;
        }


        // Now that we have an intersection, intersection information: 
        // point, color, normal, uv, etc

        // multiply distance by unit vector to find the hit_point in world coord
        let v_hit_tmp = scene.camera.eye + (closest_intersection * r.direction.as_ref());
        let p_hit: Point3<f32> = Point3::new(v_hit_tmp.x, v_hit_tmp.y, v_hit_tmp.z);
        let normal = hit_primitive.unwrap().get_normal(p_hit);
        let color: Color = hit_primitive.unwrap().get_color();

        // Now that we have closest intersection, trace ray to light
        // Add small delta so the origin of the new ray does not intersect with the object
        // immediatly
        let r_to_light_orig = p_hit /*- 0.001 * r.direction.as_ref()*/;
        let r_to_light = Ray::new(r_to_light_orig, scene.light.position - r_to_light_orig); 
        let distance_to_light: f32 = distance(&scene.light.position, &r_to_light_orig);

        closest_intersection = f32::MAX;
        for el in &scene.primitives {
            let intersection = el.intersect(&r_to_light);    

            match intersection {
                Some(x) => {
                    // WARN CAN'T HAVE SPHERE SINCE THEY HAVE 2 INTERSECTION POINT
                    if x >= 0.1 && x < closest_intersection { 
                        closest_intersection = x;
                    }
                },
                None    => {},
            }
        }

        if closest_intersection > distance_to_light {
            let light_norm_dot: f32 = normal.dot(&r_to_light.direction).abs();

            avg_col.red = avg_col.red + 
                ((color.red * light_norm_dot) / (NB_RAY as f32));
            avg_col.green = avg_col.green + 
                ((color.green * light_norm_dot) / (NB_RAY as f32));
            avg_col.blue = avg_col.blue + 
                ((color.blue * light_norm_dot) / (NB_RAY as f32)); 
        }
        else {
            // println!("{}", closest_intersection);
            avg_col.red = avg_col.red + 0.0 / (NB_RAY as f32);
            avg_col.green = avg_col.green + 0.0 / (NB_RAY as f32);
            avg_col.blue = avg_col.blue + 0.0 / (NB_RAY as f32);
        }
    }

    return avg_col;
}

pub fn render(scene: Scene) {

    let w = scene.width;
    let h = scene.height;

    let img = Arc::new(Mutex::new(DynamicImage::new_rgb8(w, h)));
    let scene_ptr = Arc::new(scene);

    let (tx, rx) = mpsc::channel();

    let mut pixels: Vec<(u32, u32)> = Vec::with_capacity((w * h) as usize);
    let mut random_samples: Vec<(f32, f32)> = Vec::with_capacity(NB_RAND_SAMPLE  as usize);
    for px in 0..w {
        for py in 0..h {
            pixels.push((px, py));
        }
    }

    let mut rng = rand::thread_rng();
    let between = Range::new(0.0, 1.0);
    for _ in 0..NB_RAND_SAMPLE {
        random_samples.push((between.ind_sample(&mut rng),
                             between.ind_sample(&mut rng)));
    }

    let mut rng = thread_rng();
    rng.shuffle(&mut pixels);
    let num_cpus = num_cpus::get();
    let pixels_ptr = Arc::new(pixels);
    let random_samples_ptr = Arc::new(random_samples);

    let time_start = time::get_time().sec;

    for i in 0..num_cpus {
        let cur_scene = scene_ptr.clone();
        let cur_pixels = pixels_ptr.clone();
        let cur_random_samples = random_samples_ptr.clone();
        let cur_img = img.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            let sliced_pixels = &cur_pixels[i * (cur_pixels.len() / num_cpus)..
                                            (i + 1) * (cur_pixels.len() / num_cpus)];
            let mut cols = Vec::with_capacity(sliced_pixels.len());
            for pixel in sliced_pixels {
                let color = render_pixel(pixel.0, pixel.1, &cur_scene, &cur_random_samples);
                cols.push((pixel.0, pixel.1, color));
            }

            let mut cur_img = cur_img.lock().unwrap();
            for c in cols {
                cur_img.put_pixel(c.0, c.1,
                                  Rgba::to_rgba(&c.2.to_rgba()));
            }

            tx.send(()).unwrap();
        });
    }
    
    for _ in 0..num_cpus {
        rx.recv().unwrap();
    }

    let time_elapsed = time::get_time().sec - time_start;
    if time_elapsed > 0 {
        println!("Rendered in {} seconds", time_elapsed);
        println!("Throughput {}M ray/s", 
            (((h * w) as u64 * (NB_RAY) as u64) / 1000000) / time_elapsed as u64);
    }
 
    println!("Writting image to disk");
    let ref mut fout = File::create(&Path::new("output.png")).unwrap();
    let img = img.lock().unwrap();
    img.save(fout, image::PNG).unwrap();

}

fn main() {

    let args: Vec<String> = env::args().collect();
    println!("Building scene");

    let mut primitives: Vec<Primitive> = Vec::new();
    // let spheres = gen_random_spheres();
    // let triangles = gen_random_triangles();
    let ground = create_ground();
    for argument in &args[1..] {
        let obj = import_obj(argument);
        primitives.extend(obj);
    }

    // primitives.extend(spheres);
    // primitives.extend(triangles);
    primitives.extend(ground);

    let area_light = Light {
        // position: Point3::new(0.0, 30.0, 10000.0) //todo find out light problem
        position: Point3::new(0.0, 1000.0, -100.0),
        primitives: Vec::new()
    };

    let scene = Scene {
        width: 1920,
        height: 1080,
        primitives: primitives,
        light: area_light,
        camera: Camera::new(Point3::new(0.0, 100.0, 200.0), 
                            Point3::new(0.0, 0.0, -100000.0), 
                            Vector3::new(0.0, 1.0, 0.0), 
                            288.0) //60 deg FOV
    };

    println!("Rendering...");
    render(scene);
}


