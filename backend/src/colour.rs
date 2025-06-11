use bytemuck::{Pod, Zeroable};
use zene_structs::{Vector3, Vector4};

#[derive(Debug, Copy, Clone, Default)]
pub struct Colour
{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}
unsafe impl Pod for Colour { }
unsafe impl Zeroable for Colour {}

impl Colour
{
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self
    {
        return Self { r, g, b, a };
    }
    pub fn rgb(r: u8, g: u8, b: u8) -> Self
    {
        return Self { r, g, b, a: 255 };
    }
}

impl From<Vector3<f32>> for Colour
{
    fn from(value: Vector3<f32>) -> Self
    {
        return Self::rgb((value.x * 255.0) as u8,
            (value.y * 255.0) as u8,
            (value.z * 255.0) as u8);
    }
}
impl From<Vector4<f32>> for Colour
{
    fn from(value: Vector4<f32>) -> Self
    {
        return Self::new((value.x * 255.0) as u8,
            (value.y * 255.0) as u8,
            (value.z * 255.0) as u8,
            (value.w * 255.0) as u8);
    }
}
impl From<[f32; 3]> for Colour
{
    fn from(value: [f32; 3]) -> Self
    {
        return Self::rgb((value[0] * 255.0) as u8,
            (value[1] * 255.0) as u8,
            (value[2] * 255.0) as u8);
    }
}
impl From<[f32; 4]> for Colour
{
    fn from(value: [f32; 4]) -> Self
    {
        return Self::new((value[0] * 255.0) as u8,
            (value[1] * 255.0) as u8,
            (value[2] * 255.0) as u8,
            (value[3] * 255.0) as u8);
    }
}
impl From<[u8; 3]> for Colour
{
    fn from(value: [u8; 3]) -> Self
    {
        return Self::rgb(value[0],
            value[1],
            value[2]);
    }
}
impl From<[u8; 4]> for Colour
{
    fn from(value: [u8; 4]) -> Self
    {
        return Self::new(value[0],
            value[1],
            value[2],
            value[3]);
    }
}