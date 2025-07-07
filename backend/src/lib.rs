mod fft;
use std::f64::consts::FRAC_1_SQRT_2;

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
use num::Zero;
use zene_structs::{Vector2, Vector3, Vector};

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

const SQRT_2_2_2N: f64 = 0.3826834323650897;
const SQRT_2_2_2: f64 = 0.9238795325112867;
const SNAP_DIR: [Vector2<f64>; 16] = [
    Vector2::new(1.0, 0.0),
    Vector2::new(-1.0, 0.0),
    Vector2::new(0.0, 1.0),
    Vector2::new(0.0, -1.0),
    Vector2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    Vector2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    Vector2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    Vector2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    
    Vector2::new(SQRT_2_2_2N, SQRT_2_2_2),
    Vector2::new(-SQRT_2_2_2N, SQRT_2_2_2),
    Vector2::new(-SQRT_2_2_2N, -SQRT_2_2_2),
    Vector2::new(SQRT_2_2_2N, -SQRT_2_2_2),
    Vector2::new(SQRT_2_2_2, SQRT_2_2_2N),
    Vector2::new(-SQRT_2_2_2, SQRT_2_2_2N),
    Vector2::new(-SQRT_2_2_2, -SQRT_2_2_2N),
    Vector2::new(SQRT_2_2_2, -SQRT_2_2_2N)
];

pub fn snap_point(origin: Vector2<f64>, wp: Vector2<f64>) -> Vector2<f64>
{
    let mut dist = f64::MAX;
    let mut np = Vector2::<f64>::zero();
    for dir in SNAP_DIR
    {
        // direction is normalised
        let t = (wp - origin).dot(dir);
        let p = origin + (dir * t);
        let d = wp.squared_distance(p);
        if d < dist
        {
            dist = d;
            np = p;
        }
    }
    
    return np;
}

pub trait UIWall
{
    fn set_a(&mut self, a: Vector2<f64>);
    fn set_b(&mut self, b: Vector2<f64>);
    fn get_a(&self) -> Vector2<f64>;
    fn get_b(&self) -> Vector2<f64>;
    
    fn set_a_b(&mut self, a: Vector2<f64>, b: Vector2<f64>)
    {
        self.set_a(a);
        self.set_b(b);
    }
    fn get_a_b(&self) -> (Vector2<f64>, Vector2<f64>)
    {
        return (self.get_a(), self.get_b());
    }
    
    fn shift_whole_wall(&mut self, a: Vector2<f64>)
    {
        let old = self.get_a_b();
        self.set_a_b(a, old.1 + a - old.0);
    }
    
    fn snap_wall_point(&mut self, ab: bool, wp: Vector2<f64>, old: (Vector2<f64>, Vector2<f64>), fixed_len: bool)
    {
        let origin = if ab { old.0 }
            else           { old.1 };
        
        self.set_wall_point(ab, snap_point(origin, wp), old, fixed_len);
    }
    
    fn set_wall_point(&mut self, ab: bool, mut p: Vector2<f64>, old: (Vector2<f64>, Vector2<f64>), fixed_len: bool)
    {
        // make sure reference point is what it was
        match ab
        {
            true if old.0 != self.get_a() => self.set_a(old.0),
            false if old.1 != self.get_b() => self.set_b(old.1),
            _ => ()
        }
        
        if fixed_len
        {
            let origin = match ab
            {
                true => old.0,
                false => old.1
            };
            let nd = p - origin;
            let sd = nd * (old.0.squared_distance(old.1) / nd.squared_length()).sqrt();
            p = origin + sd;
        }
        
        if ab
        {
            self.set_b(p);
            return;
        }
        
        self.set_a(p);
    }
    
    fn snap_wall_points(&mut self, ab: bool, wp: Vector2<f64>, old: (Vector2<f64>, Vector2<f64>), fixed_len: bool)
    {
        let origin = (old.0 + old.1) * 0.5;
        self.set_wall_points(ab, snap_point(origin, wp), old, fixed_len);
    }
    fn set_wall_points(&mut self, ab: bool, wp: Vector2<f64>, old: (Vector2<f64>, Vector2<f64>), fixed_len: bool)
    {
        let (mut a, mut b) = match ab
        {
            true =>
            {
                let off = wp - old.1;
                let a = old.0 - off;
                (a, wp)
            },
            false =>
            {
                let off = wp - old.0;
                let b = old.1 - off;
                (wp, b)
            }
        };
        
        if fixed_len
        {
            let nd = b - a;
            let sd = nd * (old.0.squared_distance(old.1) / nd.squared_length()).sqrt() * 0.5;
            let mid = (a + b) * 0.5;
            a = mid - sd;
            b = mid + sd;
        }
        
        self.set_a_b(a, b);
    }
}

impl UIWall for (Vector2<f64>, Vector2<f64>)
{
    #[inline]
    fn set_a(&mut self, a: Vector2<f64>)
    {
        self.0 = a;
    }
    #[inline]
    fn set_b(&mut self, b: Vector2<f64>)
    {
        self.1 = b;
    }
    #[inline]
    fn get_a(&self) -> Vector2<f64>
    {
        return self.0;
    }
    #[inline]
    fn get_b(&self) -> Vector2<f64>
    {
        return self.1;
    }
}