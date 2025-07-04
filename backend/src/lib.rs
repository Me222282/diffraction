mod fft;
pub use crate::fft::*;

mod diffraction;
pub use crate::diffraction::*;

mod repeat_until;
pub use crate::repeat_until::*;

mod lambda_zip;
pub use crate::lambda_zip::*;

mod em_env;
pub use crate::em_env::*;

mod colour;
pub use crate::colour::*;

use num::Complex;
use num::Float;
use num::traits::ConstOne;
use num::traits::ConstZero;
use num::traits::FloatConst;
use zene_structs::Vector3;

fn next_power_of_2(n: usize) -> u32
{
    return usize::BITS - n.leading_zeros();
}

pub fn dft_analysis<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &mut WCache<T>, plot: &[T]) -> Vec<Complex<T>>
{
    let power = next_power_of_2(plot.len());
    let hs = 1 << power;
    
    let mut data: Vec<Complex<T>> = RepeatUntil::new(plot, hs << 1)
        .map(|v| Complex::new(*v, T::ZERO)).collect();
    
    wn.ensure_max_power(power as usize + 1);
    
    dft(wn, &mut data);
    data.truncate(hs + 1);
    return data;
}

pub fn form_plot<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &mut WCache<T>, dft: &[Complex<T>], out_size: usize) -> Vec<Complex<T>>
{
    let len = dft.len() - 1;
    
    if !power_of_2(len)
    {
        panic!("dft must have a power of 2 length");
    }
    
    let power = next_power_of_2(len);
    let s = T::ONE / T::from(len << 1).unwrap();
    let mut data: Vec<Complex<T>> = dft.iter().copied().chain(dft.iter().skip(1).rev().skip(1).map(|c| c.conj()))
        .map(|n| n * s).collect();
    
    wn.ensure_max_power(power as usize);
    
    fft::dft(wn, &mut data);
    data.truncate(out_size);
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

    return Vector3::new(
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

pub trait IntoF32
{
    fn into_f32(self) -> f32;
}
// impl<T: Into<f32>> IntoF32 for T
// {
//     fn into_f32(self) -> f32
//     {
//         return self.into();
//     }
// }
impl IntoF32 for f32
{
    fn into_f32(self) -> f32
    {
        return self;
    }
}
impl IntoF32 for f64
{
    fn into_f32(self) -> f32
    {
        return self as f32;
    }
}
// impl IntoF32 for f128
// {
//     fn into_f32(self) -> f32
//     {
//         return self as f32;
//     }
// }