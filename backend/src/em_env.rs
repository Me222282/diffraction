use std::mem::replace;

use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float, Zero};
use zene_structs::{Vector2, Vector3};

use crate::Slit;

pub struct EMEnv<'a, T: Float>
{
    pub slits: Vec<Slit<'a, T>>,
    pub screen: (Vector2<T>, Vector2<T>)
}

impl<'a, T> EMEnv<'a, T>
    where T: Float + ConstOne + ConstZero + FloatConst + Into<f32>
{
    pub fn new(scr_a: Vector2<T>, scr_b: Vector2<T>) -> Self
    {
        return Self {
            slits: Vec::new(),
            screen: (scr_a, scr_b)
        };
    }
    
    fn lerp(&self, x: T) -> Vector2<T>
    {
        let a = self.screen.0;
        let b = self.screen.1;
        return a + ((b - a) * x);
    }
    
    pub fn generate_pattern<S>(&self, size: usize, wave_map: &[(T, Vector3<f32>)], samples: &mut [S])
        where S: From<Vector3<f32>>
    {
        let mut buffer: Vec<(T, Complex<T>)> = wave_map.iter().map(|w| (w.0, Complex::<T>::ZERO)).collect();
        
        let step = T::one() / T::from(size - 1).unwrap();
        let mut x = T::zero();
        
        for p in samples
        {
            let s_p = self.lerp(x);
            x = x + step;
            
            for s in &self.slits
            {
                s.calculate_intensity(s_p, &mut buffer);
            }
            
            let mut sample = Vector3::<f32>::zero();
            
            // sum total and clear buffer
            for ((_, c), (_, colour)) in buffer.iter_mut().zip(wave_map.iter())
            {
                let c = replace(c, Complex::<T>::ZERO);
                let i = *colour * c.norm_sqr().into();
                sample += i;
            }
            
            *p = sample.into();
        }
    }
}