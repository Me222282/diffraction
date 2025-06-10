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
    
    pub fn generate_pattern(&self, size: usize, wave_map: &[(T, Vector3<f32>)]) -> Vec<Vector3<f32>>
    {
        let mut buffer: Vec<(T, Complex<T>)> = wave_map.iter().map(|w| (w.0, Complex::<T>::ZERO)).collect();
        
        let step = T::one() / T::from(size).unwrap();
        let mut x = T::zero();
        
        let mut samples = vec![Vector3::<f32>::zero(); size];
        
        for p in &mut samples
        {
            let s_p = self.lerp(x);
            x = x + step;
            
            for s in &self.slits
            {
                s.calculate_intensity(s_p, &mut buffer);
            }
            
            // sum total and clear buffer
            for wave in buffer.iter_mut().zip(wave_map.iter())
            {
                let c = replace(&mut wave.0.1, Complex::<T>::ZERO);
                let i = wave.1.1 * c.norm().into();
                *p = *p + i;
            }
        }
        
        return samples;
    }
}