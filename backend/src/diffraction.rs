use std::fmt::Display;

use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};
use zene_structs::{Vector2, Vector};

use crate::LambdaZip;

// pub const C: f64 = 299_792_458.0;

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
    
    pub fn diffract(&self, beta_lambda: T) -> Complex<T>
        where T: ConstOne + ConstZero
    {
        if beta_lambda.is_zero()
        {
            return Complex::new(T::ONE, T::ZERO);
        }
        let beta = beta_lambda / self.lambda;
        
        let sc = beta.sin_cos();
        let sri = (sc.0 / beta).abs() * self.amplitude;
        return Complex::new(sri * sc.1, sri * sc.0);
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
    
    pub fn beta_lambda(&self, x: Vector2<T>) -> Option<T>
        where T: FloatConst + ConstZero + Display
    {
        let diff = x - self.position;
        let dir = self.direction;
        
        // outside viewing angle
        if diff.dot(dir) < T::ZERO
        {
            return None;
        }
        
        // sin of acute angle
        let sin = diff.perp_dot(dir) / diff.length();
        println!("{sin}");
        // beta = pi * d * sin(theta) / lambda
        return Some(T::PI() * self.width * sin);
    }
    
    pub fn calculate_intensity(&self, x: Vector2<T>, result: &mut [(T, Complex<T>)])
        where T: ConstOne + ConstZero + FloatConst + Display
    {
        let bl_o = self.beta_lambda(x);
        match bl_o
        {
            Some(bl) =>
            {
                for (res, wave) in LambdaZip::new(result.iter_mut(), self.waves.iter())
                {
                    *res = *res + wave.diffract(bl);
                }
            },
            None => {}
        };
    }
}