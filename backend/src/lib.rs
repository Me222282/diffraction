mod fft;
use num::Complex;
use num::Float;

pub use crate::fft::*;

mod diffraction;
pub use crate::diffraction::*;

pub fn get_waves<T: Float + From<usize>>(dft: &[Complex<T>], dt: T, speed: T) -> Box<[Wave<T>]>
{
    // skip f = 0Hz
    let waves = dft.into_iter().skip(1).enumerate().map(|c|
    {
        return Wave::<T>::new(speed / (dt * c.0.into()), c.1.norm_sqr());
    });
    
    return waves.collect::<Vec<Wave<T>>>().into_boxed_slice();
}