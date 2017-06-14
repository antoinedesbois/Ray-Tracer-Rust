
use image::{Rgba, Pixel};

pub struct Color {
    pub red : f32,
    pub green : f32,
    pub blue : f32
}

const GAMMA: f32 = 2.2;
fn gamma_encode(linear: f32) -> f32 {
    linear.powf(1.0 / GAMMA)
}


impl Color {
    pub fn new(red: f32, green: f32, blue: f32) -> Color {
        return Color { red: red, green: green, blue: blue }; 
    }
    pub fn new_black() -> Color {
        return Color { red: 0.0, green: 0.0, blue: 0.0 };
    }

    pub fn new_copy(color : &Color) -> Color {
        return Color { red: color.red, green: color.green, blue: color.blue };
    }

    pub fn to_rgba(&self) -> Rgba<u8> {
        Rgba::from_channels((gamma_encode(self.red) * 255.0) as u8,
                            (gamma_encode(self.green) * 255.0) as u8,
                            (gamma_encode(self.blue) * 255.0) as u8,
                            255)
    }
}
