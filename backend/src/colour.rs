use std::fmt::Display;

use bytemuck::{Pod, Zeroable};
use zene_structs::{Vector3, Vector4};

#[derive(Debug, Copy, Clone, Default)]
pub struct Colour
{
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}
unsafe impl Pod for Colour { }
unsafe impl Zeroable for Colour {}

impl Colour
{
    pub const ZERO: Colour = Colour::new(0.0, 0.0, 0.0, 0.0);
    
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self
    {
        return Self { r, g, b, a };
    }
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self
    {
        return Self { r, g, b, a: 1.0 };
    }
}

impl Display for Colour
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return write!(f, "r:{}, g:{}, b:{}, a:{}", self.r, self.g, self.b, self.a);
    }
}

impl From<Vector3> for Colour
{
    fn from(value: Vector3) -> Self
    {
        return Self::rgb(value.x,
            value.y,
            value.z);
    }
}
impl From<Vector4> for Colour
{
    fn from(value: Vector4) -> Self
    {
        return Self::new(value.x,
            value.y,
            value.z,
            value.w);
    }
}
impl From<[f32; 3]> for Colour
{
    fn from(value: [f32; 3]) -> Self
    {
        return Self::rgb(value[0],
            value[1],
            value[2]);
    }
}
impl From<[f32; 4]> for Colour
{
    fn from(value: [f32; 4]) -> Self
    {
        return Self::new(value[0],
            value[1],
            value[2],
            value[3]);
    }
}