use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};

pub fn compute_nth_roots<T: Float + ConstOne + ConstZero + FloatConst>(n: usize) -> Box<[Complex<T>]>
{
    let hn = n.div_ceil(2);
    let mut result = vec![Complex::<T>::ZERO; hn];
    let mut w = Complex::<T>::ONE;
    let wn = Complex::cis(T::TAU() / T::from(n).unwrap());
    result[0] = w;
    
    for x in &mut result
    {
        w = w * wn;
        *x = w;
    }
    
    return result.into_boxed_slice();
}

// pub fn dft<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
// {
//     fft_recursive(wn, y);
// }

/// `y.len()`must be a power of 2
pub fn fft_recursive<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    let n = y.len();
    if n == 1 { return; }
    let hn = n / 2;
    fft_recursive(wn, &mut y[0..hn]);
    fft_recursive(wn, &mut y[hn..n]);
    
    for i in 0..hn
    {
        let i2 = i + hn;
        let y0 = y[i];
        let y1 = y[i2];
        
        let w = wn[i];
        y[i] = y0 + (w * y1);
        y[i2] = y0 - (w * y1);
    }
}
/// `y.len()`must be a power of 2
pub fn fft_recursive_v2<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    let n = y.len();
    if n == 1 { return; }
    let hn = n / 2;
    fft_recursive(wn, &mut y[0..hn]);
    fft_recursive(wn, &mut y[hn..n]);
    
    for i in 0..hn
    {
        unsafe
        {
            let i2 = i + hn;
            let y0 = y.get_unchecked(i).clone();
            let y1 = y.get_unchecked(i2).clone();
            
            let w = wn.get_unchecked(i);
            let wy = w * y1;
            *y.get_unchecked_mut(i) = y0 + wy;
            *y.get_unchecked_mut(i2) = y0 - wy;
        }
    }
}

/// `y.len()`must be a power of 2
pub fn fft_iterative<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    let n = y.len();
    let mut s = 2;
    let mut hs = 1;
    while s <= n
    {
        for i in (0..n).step_by(s)
        {
            for k in 0..hs
            {
                let j1 = k + i;
                let j2 = j1 + hs;
                let y0 = y[j1];
                let y1 = y[j2];
                
                let w = wn[k];
                y[j1] = y0 + (w * y1);
                y[j2] = y0 - (w * y1);
            }
        }
        
        s *= 2;
        hs *= 2;
    }
}

/// `y.len()`must be a power of 2
pub fn fft_iterative_v2<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    let n = y.len();
    let mut s = 2;
    let mut hs = 1;
    while s <= n
    {
        for i in (0..n).step_by(s)
        {
            for k in 0..hs
            {
                unsafe
                {
                    let j1 = k + i;
                    let j2 = j1 + hs;
                    let y0 = y.get_unchecked(j1).clone();
                    let y1 = y.get_unchecked(j2).clone();
                    
                    let w = wn.get_unchecked(k);
                    let wy = w * y1;
                    *y.get_unchecked_mut(j1) = y0 + wy;
                    *y.get_unchecked_mut(j2) = y0 - wy;
                }
            }
        }
        
        hs = s;
        s <<= 1;
    }
}
/// `y.len()`must be a power of 2
pub fn fft_iterative_v3<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    let n = y.len();
    let mut old = 0;
    let mut current = 2;
    while current <= n
    {
        let x = current & (!old);
        let mut hs = 1;
        let mut s = 2;
        while s <= x
        {
            let i = current - s;
            for k in 0..hs
            {
                unsafe
                {
                    let j1 = k + i;
                    let j2 = j1 + hs;
                    let y0 = y.get_unchecked(j1).clone();
                    let y1 = y.get_unchecked(j2).clone();
                    
                    let w = wn.get_unchecked(k);
                    let wy = w * y1;
                    *y.get_unchecked_mut(j1) = y0 + wy;
                    *y.get_unchecked_mut(j2) = y0 - wy;
                }
            }
            hs = s;
            // s <<= 1;
            s *= 2;
        }
        
        old = current;
        current += 2;
    }
}