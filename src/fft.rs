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
    fft_recursive(y.len(), wn, y);
}

fn fft_recursive<T: Float>(n: usize, wn: &[Complex<T>], y: &mut [Complex<T>])
{
    if n == 1 { return; }
    let hn = n / 2;
    fft_recursive(hn, wn, &mut y[0..hn]);
    fft_recursive(hn, wn, &mut y[hn..n]);
    
    for i in 0..hn
    {
        let i2 = i + hn;
        let y0 = y[i];
        let y1 = y[i2];
        
        let w = wn[i];
        y[i] = y0 + (w * y1);
        y[i2] = y0 + (w * y1);
    }
}