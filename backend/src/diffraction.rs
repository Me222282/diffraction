use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};
use zene_structs::{Vector2, Vector};

use crate::LambdaZip;

#[derive(Debug, Clone, Default)]
pub struct Wave<T: Float>
{
    pub amplitude: T,
    pub lambda: T
}

impl<T: Float> Wave<T>
{
    pub fn new(wavelength: T, amplitude: T) -> Self
    {
        return Self { amplitude, lambda: wavelength };
    }
    
    pub fn diffract(&self, diff_args: (T, T)) -> Complex<T>
        where T: ConstOne + ConstZero + FloatConst
    {
        let rec = T::ONE / self.lambda;
        let phase = T::TAU() * (diff_args.1 % self.lambda) * rec;
        
        if diff_args.0.is_zero()
        {
            return Complex::from_polar(T::ONE, phase);
        }
        let beta = diff_args.0 * rec;
        
        let sin = beta.sin();
        let sri = (sin / beta).abs() * self.amplitude;
        return Complex::from_polar(sri, phase);
    }
}

pub struct Slit<'a, T: Float>
{
    pub width: T,
    pub position: Vector2<T>,
    direction: Vector2<T>,
    pub waves: &'a [Wave<T>]
}

impl<'a, T: Float> Slit<'a, T>
{
    pub fn new(width: T, position: Vector2<T>, direction: Vector2<T>, waves: &'a [Wave<T>]) -> Self
    {
        return Self {
            width,
            position,
            direction: direction.normalised(),
            waves
        };
    }
    
    pub fn get_direction(&self) -> Vector2<T>
    {
        return self.direction;
    }
    pub fn set_direction(&mut self, direction: Vector2<T>)
    {
        self.direction = direction.normalised();
    }
    
    pub fn diff_args(&self, x: Vector2<T>) -> Option<(T, T)>
        where T: FloatConst + ConstZero
    {
        let diff = x - self.position;
        let dir = self.direction;
        
        // outside viewing angle
        if diff.dot(dir) < T::ZERO
        {
            return None;
        }
        
        let len = diff.length();
        // sin of acute angle
        let sin = diff.perp_dot(dir) / len;
        // beta = pi * d * sin(theta) / lambda
        return Some((T::PI() * self.width * sin, len));
    }
    
    pub fn calculate_intensity(&self, x: Vector2<T>, result: &mut [(T, Complex<T>)])
        where T: ConstOne + ConstZero + FloatConst
    {
        let args_o = self.diff_args(x);
        match args_o
        {
            Some(args) =>
            {
                for (res, wave) in LambdaZip::new(result.iter_mut(), self.waves.iter())
                {
                    *res = *res + wave.diffract(args);
                }
            },
            None => {}
        };
    }
}