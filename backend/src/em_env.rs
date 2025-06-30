use std::mem::replace;

use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float, NumCast, Zero};
use zene_structs::{Vector2, Vector3};

use crate::{IntoF32, Slit};

#[derive(Debug, Clone)]
pub struct EMEnv<T: Float>
{
    pub screen: (Vector2<T>, Vector2<T>)
}

impl<T> Default for EMEnv<T>
    where T: Float + ConstZero
{
    fn default() -> Self
    {
        return Self {
            screen: (Vector2::zero(), Vector2::zero())
        };
    }
}

impl<T> EMEnv<T>
    where T: Float + ConstOne + ConstZero + FloatConst + IntoF32
{
    pub fn new(scr_a: Vector2<T>, scr_b: Vector2<T>) -> Self
    {
        return Self {
            screen: (scr_a, scr_b)
        };
    }
    
    fn lerp(&self, x: T) -> Vector2<T>
    {
        let a = self.screen.0;
        let b = self.screen.1;
        return a + ((b - a) * x);
    }
    
    pub fn generate_pattern<S>(&self, slits: &[Slit<'_, T>], wave_map: &[(T, Vector3<f32>)], samples: &mut [S])
        where S: From<Vector3<f32>>
    {
        let mut buffer: Vec<(T, Complex<T>)> = wave_map.iter().map(|w| (w.0, Complex::<T>::ZERO)).collect();
        
        let step = T::one() / <T as NumCast>::from(samples.len() - 1).unwrap();
        let mut x = T::zero();
        
        for p in samples
        {
            let s_p = self.lerp(x);
            x = x + step;
            
            for s in slits
            {
                s.calculate_intensity(s_p, &mut buffer);
            }
            
            let mut sample = Vector3::<f32>::zero();
            
            // sum total and clear buffer
            for ((_, c), (_, colour)) in buffer.iter_mut().zip(wave_map.iter())
            {
                let c = replace(c, Complex::<T>::ZERO);
                let i = *colour * c.norm_sqr().into_f32();
                sample += i;
            }
            
            *p = sample.into();
        }
    }
}