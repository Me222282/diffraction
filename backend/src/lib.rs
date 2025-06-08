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
use zene_structs::Vector3;

pub fn get_waves<T: Float + From<usize>>(dft: &[Complex<T>], dt: T, speed: T) -> Box<[Wave<T>]>
{
    // skip f = 0Hz
    let waves = dft.into_iter().skip(1).enumerate().map(|c|
    {
        return Wave::<T>::new(speed / (dt * c.0.into()), c.1.norm_sqr());
    });
    
    return waves.collect::<Vec<Wave<T>>>().into_boxed_slice();
}

fn next_power_of_2(n: usize) -> u32
{
    return usize::BITS - n.leading_zeros();
}

pub fn dft_analysis<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &mut WCache<T>, plot: &[T]) -> Vec<Complex<T>>
{
    // at least 1 loop of the plot, but no more than 2
    let power = next_power_of_2(plot.len());
    let hs = 1 << power;
    
    let mut data: Vec<Complex<T>> = RepeatUntil::new(plot, hs << 1)
        .map(|v| Complex::new(*v, T::ZERO)).collect();
        // .chain(std::iter::repeat(Complex::<T>::ZERO).take(hs)).collect();
    
    wn.ensure_max_power(power as usize + 1);
    
    dft(wn, &mut data);
    data.truncate(hs);
    return data;
}

/// stolen function
pub fn wave_length_colour(wl: f32, gamma: f32) -> Vector3<f32>
{
    let factor;
    let r;
    let g;
    let b;
    
    match wl
    {
        380.0..440.0 =>
        {
            r = -(wl - 440.0) / (440.0 - 380.0);
            g = 0.0;
            b = 1.0;
        },
        440.0..490.0 =>
        {
            r = 0.0;
            g = (wl - 440.0) / (490.0 - 440.0);
            b = 1.0;
        },
        490.0..510.0 =>
        {
            r = 0.0;
            g = 1.0;
            b = -(wl - 510.0) / (510.0 - 490.0);
        },
        510.0..580.0 =>
        {
            r = (wl - 510.0) / (580.0 - 510.0);
            g = 1.0;
            b = 0.0;
        },
        580.0..645.0 =>
        {
            r = 1.0;
            g = -(wl - 645.0) / (645.0 - 580.0);
            b = 0.0;
        },
        645.0..781.0 =>
        {
            r = 1.0;
            g = 0.0;
            b = 0.0;
        },
        _ =>
        {
            r = 0.0;
            g = 0.0;
            b = 0.0;
        }
    }

    // Let the intensity fall off near the vision limits
    
    const LF: f32 = 0.7 / (420.0 - 380.0);
    const HF: f32 = 0.7 / (780.0 - 700.0);
    
    match wl
    {
        380.0..420.0 => factor = 0.3 + LF * (wl - 380.0),
        420.0..701.0 => factor = 1.0,
        701.0..781.0 => factor = 0.3 + HF * (780.0 - wl),
        _ => factor = 0.0
    }

    return Vector3::<f32>::new(
        match r
        {
            0.0 => 0.0,
            _ => (r * factor).powf(gamma)
        },
        match g
        {
            0.0 => 0.0,
            _ => (g * factor).powf(gamma)
        },
        match b
        {
            0.0 => 0.0,
            _ => (b * factor).powf(gamma)
        }
    );
}