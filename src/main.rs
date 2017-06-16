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

use tracer::utils::scene::Scene;
use tracer::utils::color::Color;
use tracer::utils::ray::Ray;
use tracer::primitives::Intersectable;
use tracer::utils::intersection::Intersection;

use image::{DynamicImage, Rgba, GenericImage, Pixel};
use nalgebra::{Point3, Vector3};

use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex, mpsc};
use std::fs::File;
use std::path::Path;
use std::f32;
use std::thread;

const NB_RAY: u32 = 10000; //Per pixel
const NB_RAND_SAMPLE: u32 = 2000000;

fn gen_random_spheres() -> Vec<Primitive> {

    let mut primitives: Vec<Primitive> = Vec::new();
    let mut rng = rand::thread_rng();
    let between_0_1 = Range::new(0.0, 1.0);
    // let between_0_500 = Range::new(0.0, 500.0);
    let between_n500_500 = Range::new(-500.0, 500.0);

    for _ in 0..10 {
        primitives.push(
            Primitive::Sphere(
                Sphere::new(
                        between_0_1.ind_sample(&mut rng), 
                        Point3::new(between_n500_500.ind_sample(&mut rng), 
                                    between_n500_500.ind_sample(&mut rng), 
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

fn gen_random_triangles() -> Vec<Primitive> {

    let mut primitives: Vec<Triangle> = Vec::new();
    let mut rng = rand::thread_rng();
    let between_0_1 = Range::new(0.0, 1.0);
    // let between_0_500 = Range::new(0.0, 500.0);
    let between_n500_500 = Range::new(-500.0, 500.0);

    for _ in 0..10 {
        primitives.push(
            Primitive::Triangle(
                Triangle::new(Point3::new(between_n500_500.ind_sample(&mut rng), 
                                          between_n500_500.ind_sample(&mut rng), 
                                          -1000.0),
                              Point3::new(between_n500_500.ind_sample(&mut rng), 
                                          between_n500_500.ind_sample(&mut rng), 
                                          -1000.0),
                              Point3::new(between_n500_500.ind_sample(&mut rng), 
                                          between_n500_500.ind_sample(&mut rng), 
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

pub fn render_pixel(px: u32, py: u32, scene: &Scene, random_samples: &Vec<(f32, f32)>) -> Color {
    //Create ray and trace
    let o_x: f32 = px as f32;
    let o_y: f32 = py as f32;
    let w: f32 = scene.width as f32;
    let h: f32 = scene.height as f32;
    let mut avg_col = Color::new_black();

    for i in 0..NB_RAY {

        let r = Ray {
            origin: Point3::new(
                o_x - w/2.0 + 
                    random_samples[((px * scene.width + py + i) % NB_RAND_SAMPLE) as usize].0,
                o_y - h/2.0 + 
                    random_samples[((px * scene.width + py + i) % NB_RAND_SAMPLE) as usize].1, 
                0.0),
            direction: Vector3::new(0.0, 0.0, -1.0)
        };

        let mut closest_intersection = 
            Intersection {
                color: Color::new_black(),
                time: f32::MAX
            };

        for el in &scene.primitives {
            let intersection = el.intersect(&r);    

            match intersection {
                Some(x) => {
                    if x.time < closest_intersection.time {
                        closest_intersection = x;
                    }
                },
                None    => {},
            }
        }

        avg_col.red = avg_col.red + (closest_intersection.color.red / NB_RAY as f32);
        avg_col.green = avg_col.green + (closest_intersection.color.green / NB_RAY as f32);
        avg_col.blue = avg_col.blue + (closest_intersection.color.blue / NB_RAY as f32); 

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
                cur_img.put_pixel(c.0, (h -1) - c.1,
                                  Rgba::to_rgba(&c.2.to_rgba()));
            }

            tx.send(()).unwrap();
        });
    }
    
    for _ in 0..num_cpus {
        rx.recv().unwrap();
    }

    let time_elapsed = time::get_time().sec - time_start;
    println!("Rendered in {} seconds", time_elapsed);
    println!("Throughput {}M ray/s", 
        (((h * w) as u64 * (NB_RAY) as u64) / 1000000) / time_elapsed as u64);
 
    println!("Writting image to disk");
    let ref mut fout = File::create(&Path::new("output.png")).unwrap();
    let img = img.lock().unwrap();
    img.save(fout, image::PNG).unwrap();

}

fn main() {

    println!("Building scene");

    let scene = Scene {
        width: 1920,
        height: 1080,
        primitives: gen_random_triangles()
    };

    println!("Rendering...");
    render(scene);
}


