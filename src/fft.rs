use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};

pub fn compute_nth_roots<T: Float + ConstOne + ConstZero + FloatConst>(n: usize) -> Box<[Complex<T>]>
{
    let hn = n.div_ceil(2);
    let mut result = Vec::<Complex<T>>::with_capacity(hn);
    let mut w = Complex::<T>::ONE;
    let wn = Complex::cis(T::TAU() / T::from(n).unwrap());
    result[0] = w;
    
    for i in 1..hn
    {
        w = w * wn;
        result[i] = w;
    }
    
    return result.into_boxed_slice();
}

pub fn dft<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    fft_recursive(wn, y);
}

/// `y.len()`must be a power of 2
fn fft_recursive<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
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
fn fft_iterative<T: Float>(wn: &[Complex<T>], y: &mut [Complex<T>])
{
    let n = y.len();
    let mut s = 2;
    let mut hs = 1;
    while s <= n
    {
        for i in (0..n).step_by(s)
        {
            for j in i..(i + hs)
            {
                let j2 = j + hs;
                let y0 = y[j];
                let y1 = y[j2];
                
                let w = wn[j];
                y[j] = y0 + (w * y1);
                y[j2] = y0 - (w * y1);
            }
        }
        
        s *= 2;
        hs *= 2;
    }
}