extern crate image;
extern crate nalgebra;
extern crate alga;
extern crate rand;
extern crate num_cpus;
extern crate threadpool;
extern crate pbr;

mod tracer;

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

const NB_RAY: u32 = 100; //Per pixel

fn gen_random_sample(lower_bound: f32, upper_bound: f32) -> f32 {
    let mut rng = rand::thread_rng();
    let between = Range::new(lower_bound, upper_bound);

    return between.ind_sample(&mut rng);
}

fn quadratic_solution(a: f32, b: f32, c: f32) -> (f32, f32) {
    return (((- b + (b*b - 4.0*a*c).sqrt() ) / (2.0 * a)),
            ((- b - (b*b - 4.0*a*c).sqrt() ) / (2.0 * a)));
}

pub fn render_pixel(px: u32, py: u32, scene: std::sync::Arc<Scene>) -> Color {
    //Create ray and trace
    let o_x: f32 = px as f32;
    let o_y: f32 = py as f32;
    let w: f32 = scene.width as f32;
    let h: f32 = scene.height as f32;
    let mut avg_col = Color::new_black();

    for _ in 0..NB_RAY {

        let r = Ray {
            origin: Point3::new(o_x - w/2.0 + gen_random_sample(0.0, 1.0), 
                                o_y - h/2.0 + gen_random_sample(0.0, 1.0), 
                                0.0),
            direction: Vector3::new(0.0, 0.0, -1.0)
        };

        let intersection = sphere.intersect(r);

        match intersection {
            Some(x) => {
                avg_col.red = avg_col.red + (x.color.red / NB_RAY as f32);
                avg_col.green = avg_col.green + (x.color.green / NB_RAY as f32);
                avg_col.blue = avg_col.blue + (x.color.blue / NB_RAY as f32);       
            },
            None    => {},
        }

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
    println!("Building scene");
    let scene = Scene {
        width: 1920,
        height: 1080,
        sphere: 
            Sphere {
                radius: 500.0,
                origin: Point3::new(0.0, 0.0, 0.0),
                color: Color::new(1.0, 0.0, 0.0)
            }
    };

    println!("Rendering...");
    render(scene);
    
}


