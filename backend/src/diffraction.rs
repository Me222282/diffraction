use std::f64::consts::PI;

use num::Complex;

pub const C: f64 = 299_792_458.0;

pub struct Wave
{
    pub intensity: f64,
    pub lambda: f64
}

impl Wave
{
    pub fn new(freq: f64, amp: f64) -> Self
    {
        return Self { intensity: amp, lambda: C / freq };
    }
    
    pub fn diffract(self, beta_lambda: f64) -> Complex<f64>
    {
        if beta_lambda == 0.0
        {
            return Complex::new(self.intensity, 0.0);
        }
        let b = beta_lambda * self.lambda;
        
        let sc = b.sin_cos();
        
        let sri = sc.0 / b;
        return Complex::new(sri * sc.1, sri * sc.0);
    }
}

pub struct Slit
{
    pub width: f64,
    pub distance: f64
}

impl Slit
{
    pub fn new(width: f64, distance: f64) -> Self
    {
        return Self { width, distance };
    }
    
    pub fn beta_lambda(self, x: f64) -> f64
    {
        let d2 = self.distance * self.distance;
        let x2 = x * x;
        return (PI * x * self.width) / (d2 + x2).sqrt();
    }
}