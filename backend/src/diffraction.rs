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
    
    pub fn diffract(&self, beta_lambda: f64) -> Complex<f64>
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
    pub position: f64
}

impl Slit
{
    pub fn new(width: f64, position: f64) -> Self
    {
        return Self { width, position };
    }
    
    pub fn beta_lambda(&self, x: f64, d: f64) -> f64
    {
        let x = x - self.position;
        
        let d2 = d * d;
        let x2 = x * x;
        return (PI * x * self.width) / (d2 + x2).sqrt();
    }
}

pub fn calculate_intensity(slits: &[Slit], waves: &[Wave], x: f64, dist: f64, result: &mut [Complex<f64>])
{
    for s in slits
    {
        let bl = s.beta_lambda(x, dist);
        for (res, wave) in result.iter_mut().zip(waves)
        {
            *res += wave.diffract(bl);
        }
    }
}