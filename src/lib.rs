pub use uuid::Uuid;
pub use serde_json;
pub use serde::{Deserialize,Serialize};
pub use rand;


pub mod element;
pub mod event;
pub mod scene;
pub mod instance;
pub mod sprite;
pub mod core;

#[derive(Copy,Clone,Debug,Deserialize,Serialize)]
pub struct V2<T: Clone + Copy> {
    pub x: T,
    pub y: T
}


impl<T: Clone + Copy> From<[T;2]> for V2<T> {
    fn from(value: [T;2]) -> Self {
        Self {
            x: value[0],
            y: value[1]
        }
    }
}

impl TryFrom<serde_json::Value> for V2<f32> {
    type Error = String;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        use serde_json::*;
        if let Value::Array(arr) = value {
            if arr.len() < 2 {
                Err("Array not long enough".to_string())
            } else {
                let x = if let Value::Number(num) = &arr[0] { num.as_f64() } else { None };
                let y = if let Value::Number(num) = &arr[1] { num.as_f64() } else { None };
                if !(x.is_none() || y.is_none()) {
                    return Ok(V2 { x: x.unwrap() as f32, y: y.unwrap() as f32})
                } else {
                    Err("Not a number".to_string())
                }
            } 
        } else {
            Err("Not array".to_string())
        }
    }
}

impl<T: Clone + Copy + std::ops::Mul<Output = T>> std::ops::Mul<T> for V2<T> {
    type Output = V2<T>;
    fn mul(self, rhs: T) -> Self::Output {Self {x: self.x*rhs, y: self.y*rhs}}
}

impl<T: Clone + Copy + std::ops::Add<Output = T>> std::ops::Add<Self> for V2<T> {
    type Output = V2<T>;
    fn add(self, rhs: Self) -> Self::Output {Self {x: self.x+rhs.x, y: self.y+rhs.y}}
}
impl<T: Clone + Copy + std::ops::Add<Output = T>> std::ops::Add<T> for V2<T> {
    type Output = V2<T>;
    fn add(self, rhs: T) -> Self::Output {Self {x: self.x+rhs, y: self.y+rhs}}
}

impl<T: Clone + Copy + std::ops::Sub<Output = T>> std::ops::Sub<Self> for V2<T> {
    type Output = V2<T>;
    fn sub(self, rhs: Self) -> Self::Output {Self {x: self.x-rhs.x, y: self.y-rhs.y}}
}
impl<T: Clone + Copy + std::ops::Sub<Output = T>> std::ops::Sub<T> for V2<T> {
    type Output = V2<T>;
    fn sub(self, rhs: T) -> Self::Output {Self {x: self.x-rhs, y: self.y-rhs}}
}

impl<T: Clone + Copy + std::ops::Div<Output = T>> std::ops::Div<T> for V2<T> {
    type Output = V2<T>;
    fn div(self, rhs: T) -> Self::Output {Self {x: self.x/rhs, y: self.y/rhs}}
}


impl<T: Clone + Copy + Default> std::default::Default for V2<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default()
        }
    }
}



#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32;3],
    pub tex_coords: [f32;2],
    rotation: f32
}


glium::implement_vertex!(Vertex, pos, tex_coords, rotation);