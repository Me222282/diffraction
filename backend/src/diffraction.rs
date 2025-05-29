use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};

// pub const C: f64 = 299_792_458.0;

pub struct Wave<T: Float>
{
    pub intensity: T,
    pub lambda: T
}

impl<T: Float> Wave<T>
{
    pub fn new(wavelength: T, intensity: T) -> Self
    {
        return Self { intensity, lambda: wavelength };
    }
}
impl<T: Float + ConstOne + ConstZero> Wave<T>
{
    pub fn diffract(&self, beta_lambda: T) -> Complex<T>
    {
        if beta_lambda.is_zero()
        {
            return Complex::new(T::ONE, T::ZERO);
        }
        let b = beta_lambda * self.lambda;
        
        let sc = b.sin_cos();
        
        let sri = (sc.0 / b).abs();
        return Complex::new(sri * sc.1, sri * sc.0);
    }
}

pub struct Slit<T: Float>
{
    pub width: T,
    pub position: T
}

impl<T: Float + FloatConst> Slit<T>
{
    pub fn new(width: T, position: T) -> Self
    {
        return Self { width, position };
    }
    
    pub fn beta_lambda(&self, x: T, d: T) -> T
    {
        let x = x - self.position;
        
        let d2 = d * d;
        let x2 = x * x;
        return (T::PI() * x * self.width) / (d2 + x2).sqrt();
    }
}

pub fn calculate_intensity<T: Float + ConstOne + ConstZero + FloatConst>(slits: &[Slit<T>], waves: &[Wave<T>], x: T, dist: T, result: &mut [Complex<T>])
{
    for s in slits
    {
        let bl = s.beta_lambda(x, dist);
        for (res, wave) in result.iter_mut().zip(waves)
        {
            *res = *res + wave.diffract(bl);
        }
    }
}