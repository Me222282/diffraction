use std::mem::swap;

use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};

#[derive(Debug, Clone, Default)]
pub struct WCache<T>
{
    nth_roots: Vec<Box<[Complex<T>]>>,
    invert: bool
}
impl<T: Float + ConstOne + ConstZero + FloatConst> WCache<T>
{
    pub fn new(invert: bool) -> Self
    {
        let nth_roots = vec![compute_nth_roots(2, invert)];
        return Self { nth_roots, invert };
    }
    pub fn get_nth_roots(&self, power: usize) -> &[Complex<T>]
    {
        return &self.nth_roots[power - 1];
    }
    pub unsafe fn get_nth_roots_unchecked(&self, power: usize) -> &[Complex<T>]
    {
        return self.nth_roots.get_unchecked(power - 1);
    }
    pub fn ensure_max_power(&mut self, power: usize)
    {
        let len = self.nth_roots.len();
        if len >= power { return; }
        
        for x in len..power
        {
            self.nth_roots.push(compute_nth_roots(2 << x, self.invert));
        }
    }
    pub fn set_invert(&mut self, invert: bool)
    {
        if self.invert == invert { return; }
        
        // recalculate all data for inverted/normal fft
        let power = self.nth_roots.len();
        self.nth_roots.clear();
        self.ensure_max_power(power);
    }
}

fn compute_nth_roots<T: Float + ConstOne + ConstZero + FloatConst>(n: usize, invert: bool) -> Box<[Complex<T>]>
{
    let hn = n.div_ceil(2);
    let mut result = vec![Complex::<T>::ZERO; hn];
    // let mut result = vec![Complex::<T>::ZERO; n];
    let mut w = Complex::<T>::ONE;
    let wn = match invert
    {
        false => Complex::cis(T::TAU() / T::from(n).unwrap()),
        true => Complex::cis(-T::TAU() / T::from(n).unwrap())
    };
    result[0] = w;
    
    for x in result.iter_mut().skip(1)
    {
        w = w * wn;
        *x = w;
    }
    
    return result.into_boxed_slice();
}

pub(super) fn power_of_2(n: usize) -> bool
{
    return (n & (n - 1)) == 0;
}

pub fn dft<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>])
{
    if !power_of_2(y.len())
    {
        panic!("y must have a power of 2 length");
    }
    // dft_test(wn, y);
    fft_iterative_v2(wn, y, (usize::BITS - y.len().leading_zeros() - 1) as usize);
    // fft_recursive(wn, y, (usize::BITS - y.len().leading_zeros() - 1) as usize);
}

/// `y.len()`must be a power of 2
// pub fn fft_recursive<T: Float + ConstOne + ConstZero + FloatConst>(
//     wn: &WCache<T>, y: &mut [Complex<T>], p: usize)
// {
//     let n = y.len();
//     if n == 1 { return; }
//     let hn = n >> 1;
//     fft_recursive(wn, &mut y[0..hn], p - 1);
//     fft_recursive(wn, &mut y[hn..n], p - 1);
    
//     let w = wn.get_nth_roots(p);
    
//     for i in 0..hn
//     {
//         let i2 = i + hn;
//         let y0 = y[i];
//         let y1 = y[i2];
        
//         let w = w[i];
//         y[i] = y0 + (w * y1);
//         y[i2] = y0 - (w * y1);
//     }
// }
// /// `y.len()`must be a power of 2
// pub fn fft_recursive_v2<T: Float + ConstOne + ConstZero + FloatConst>(
//     wn: &WCache<T>, y: &mut [Complex<T>], p: usize)
// {
//     let n = y.len();
//     if n == 1 { return; }
//     let hn = n >> 1;
//     fft_recursive_v2(wn, &mut y[0..hn], p - 1);
//     fft_recursive_v2(wn, &mut y[hn..n], p - 1);
    
//     unsafe
//     {
//         let w = wn.get_nth_roots_unchecked(p);
        
//         for i in 0..hn
//         {
//             let i2 = i + hn;
//             let y0 = y.get_unchecked(i).clone();
//             let y1 = y.get_unchecked(i2).clone();
            
//             let w = w.get_unchecked(i);
//             let wy = w * y1;
//             *y.get_unchecked_mut(i) = y0 + wy;
//             *y.get_unchecked_mut(i2) = y0 - wy;
//         }
//     }
// }

// /// `y.len()`must be a power of 2
// pub fn fft_iterative<T: Float + ConstOne + ConstZero + FloatConst>(
//     wn: &WCache<T>, y: &mut [Complex<T>])
// {
//     let n = y.len();
//     let mut p = 1;
//     let mut s = 2;
//     let mut hs = 1;
//     while s <= n
//     {
//         let w = wn.get_nth_roots(p);
        
//         for i in (0..n).step_by(s)
//         {
//             for k in 0..hs
//             {
//                 let j1 = k + i;
//                 let j2 = j1 + hs;
//                 let y0 = y[j1];
//                 let y1 = y[j2];
                
//                 let w = w[k];
//                 y[j1] = y0 + (w * y1);
//                 y[j2] = y0 - (w * y1);
//             }
//         }
        
//         hs = s;
//         s <<= 1;
//         p += 1;
//     }
// }

/// `y.len()`must be a power of 2
pub fn fft_iterative_v2<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>], power: usize)
{
    let n = y.len();
    
    let mut buffer = vec![Complex::<T>::ZERO; y.len()];
    let mut din = y;
    let mut dout: &mut [Complex<T>] = buffer.as_mut();
    
    let hn = n >> 1;
    let mut hs = hn;
    let mut hj = 1;
    let mut p = 1;
    let mut m = power;
    while hs >= 1
    {
        unsafe
        {
            let w = wn.get_nth_roots_unchecked(p);
        
            for j in 0..hj
            {
                let w = w.get_unchecked(j);
                let k = j << m;
                let j = k >> 1;
                for i in 0..hs
                {
                    let j1 = i + k;
                    let j2 = j1 + hs;
                    let y0 = din.get_unchecked(j1).clone();
                    let y1 = din.get_unchecked(j2).clone();
                    
                    let wy = w * y1;
                    let j3 = i + j;
                    *dout.get_unchecked_mut(j3) = y0 + wy;
                    *dout.get_unchecked_mut(j3 + hn) = y0 - wy;
                }
            }
        }
        
        hj += hj;
        hs >>= 1;
        p += 1;
        m -= 1;
        swap(&mut din, &mut dout);
    }
    
    if p % 2 == 0
    {
        dout.copy_from_slice(din);
    }
}
/// `y.len()`must be a power of 2
// pub fn fft_iterative_v3<T: Float + ConstOne + ConstZero + FloatConst>(
//     wn: &WCache<T>, y: &mut [Complex<T>])
// {
//     let n = y.len();
//     let mut old = 0;
//     let mut current = 2;
//     while current <= n
//     {
//         let x = current & (!old);
//         let mut hs = 1;
//         let mut s = 2;
//         let mut p = 1;
//         while s <= x
//         {
//             unsafe
//             {
//                 let w = wn.get_nth_roots_unchecked(p);
            
//                 let i = current - s;
//                 for k in 0..hs
//                 {
//                     let j1 = k + i;
//                     let j2 = j1 + hs;
//                     let y0 = y.get_unchecked(j1).clone();
//                     let y1 = y.get_unchecked(j2).clone();
                    
//                     let w = w.get_unchecked(k);
//                     let wy = w * y1;
//                     *y.get_unchecked_mut(j1) = y0 + wy;
//                     *y.get_unchecked_mut(j2) = y0 - wy;
//                 }
//             }
//             hs = s;
//             s <<= 1;
//             // s *= 2;
//             p += 1;
//         }
        
//         old = current;
//         current += 2;
//     }
// }

fn dft_test<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>])
{
    let len = y.len();
    let power = (usize::BITS - len.leading_zeros() - 1) as usize;
    
    let n = T::ONE / T::from(len).unwrap();
    
    let out: Vec<Complex<T>> = (0..len).map(|o|
    {
        // let w: &[Complex<T>];
        // unsafe
        // {
        //     w = wn.get_nth_roots_unchecked(power);
        // }
        let k = T::from(o).unwrap();
        
        let mut sum = Complex::<T>::ZERO;
        for x in y.iter().enumerate()
        {
            // let i = (o * x.0) % len;
            // unsafe
            // {
            //     sum = sum + (*x.1 * w.get_unchecked(i));
            // }
            let w = Complex::<T>::cis(-T::TAU() * n * k * T::from(x.0).unwrap());
            sum = sum + (*x.1 * w);
        }
        
        return sum;
    }).collect();
    
    y.copy_from_slice(&out);
}