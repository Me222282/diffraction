mod fft;
pub use crate::fft::*;

mod diffraction;
pub use crate::diffraction::*;

mod repeat_until;
pub use crate::repeat_until::*;

use num::Complex;
use num::Float;
use zene_structs::ConstOne;
use zene_structs::ConstZero;
use zene_structs::FloatConst;

pub fn get_waves<T: Float + From<usize>>(dft: &[Complex<T>], dt: T, speed: T) -> Box<[Wave<T>]>
{
    // skip f = 0Hz
    let waves = dft.into_iter().skip(1).enumerate().map(|c|
    {
        return Wave::<T>::new(speed / (dt * c.0.into()), c.1.norm_sqr());
    });
    
    return waves.collect::<Vec<Wave<T>>>().into_boxed_slice();
}

pub fn next_power_of_2(n: usize) -> u32
{
    return usize::BITS - n.leading_zeros();
}

pub fn dft_analysis<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &mut WCache<T>, plot: &[T]) -> Vec<Complex<T>>
{
    // at least 1 loop of the plot, but no more than 2
    let power = next_power_of_2(plot.len());
    let mut data: Vec<Complex<T>> = RepeatUntil::new(plot, 1 << power)
        .map(|v| Complex::new(*v, T::ZERO)).collect();
    
    wn.ensure_max_power(power as usize);
    
    dft(wn, &mut data);
    return data;
}