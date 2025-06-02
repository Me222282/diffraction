use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};

#[derive(Debug, Clone, Default)]
pub struct WCache<T>
{
    nth_roots: Vec<Box<[Complex<T>]>>
}
impl<T: Float + ConstOne + ConstZero + FloatConst> WCache<T>
{
    pub fn new() -> Self
    {
        let nth_roots = vec![compute_nth_roots(2)];
        return Self { nth_roots };
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
            self.nth_roots.push(compute_nth_roots(2 << x));
        }
    }
}

fn compute_nth_roots<T: Float + ConstOne + ConstZero + FloatConst>(n: usize) -> Box<[Complex<T>]>
{
    let hn = n.div_ceil(2);
    let mut result = vec![Complex::<T>::ZERO; hn];
    let mut w = Complex::<T>::ONE;
    let wn = Complex::cis(T::TAU() / T::from(n).unwrap());
    result[0] = w;
    
    for x in result.iter_mut().skip(1)
    {
        w = w * wn;
        *x = w;
    }
    
    return result.into_boxed_slice();
}

fn power_of_2(n: usize) -> bool
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
    fft_iterative_v3(wn, y);
}

/// `y.len()`must be a power of 2
pub fn fft_recursive<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>], p: usize)
{
    let n = y.len();
    if n == 1 { return; }
    let hn = n >> 1;
    fft_recursive(wn, &mut y[0..hn], p - 1);
    fft_recursive(wn, &mut y[hn..n], p - 1);
    
    let w = wn.get_nth_roots(p);
    
    for i in 0..hn
    {
        let i2 = i + hn;
        let y0 = y[i];
        let y1 = y[i2];
        
        let w = w[i];
        y[i] = y0 + (w * y1);
        y[i2] = y0 - (w * y1);
    }
}
/// `y.len()`must be a power of 2
pub fn fft_recursive_v2<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>], p: usize)
{
    let n = y.len();
    if n == 1 { return; }
    let hn = n >> 1;
    fft_recursive_v2(wn, &mut y[0..hn], p - 1);
    fft_recursive_v2(wn, &mut y[hn..n], p - 1);
    
    unsafe
    {
        let w = wn.get_nth_roots_unchecked(p);
        
        for i in 0..hn
        {
            let i2 = i + hn;
            let y0 = y.get_unchecked(i).clone();
            let y1 = y.get_unchecked(i2).clone();
            
            let w = w.get_unchecked(i);
            let wy = w * y1;
            *y.get_unchecked_mut(i) = y0 + wy;
            *y.get_unchecked_mut(i2) = y0 - wy;
        }
    }
}

/// `y.len()`must be a power of 2
pub fn fft_iterative<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>])
{
    let n = y.len();
    let mut p = 1;
    let mut s = 2;
    let mut hs = 1;
    while s <= n
    {
        let w = wn.get_nth_roots(p);
        
        for i in (0..n).step_by(s)
        {
            for k in 0..hs
            {
                let j1 = k + i;
                let j2 = j1 + hs;
                let y0 = y[j1];
                let y1 = y[j2];
                
                let w = w[k];
                y[j1] = y0 + (w * y1);
                y[j2] = y0 - (w * y1);
            }
        }
        
        hs = s;
        s <<= 1;
        p += 1;
    }
}

/// `y.len()`must be a power of 2
pub fn fft_iterative_v2<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>])
{
    let n = y.len();
    let mut p = 1;
    let mut s = 2;
    let mut hs = 1;
    while s <= n
    {
        unsafe
        {
            let w = wn.get_nth_roots_unchecked(p);
        
            for i in (0..n).step_by(s)
            {
                for k in 0..hs
                {
                    let j1 = k + i;
                    let j2 = j1 + hs;
                    let y0 = y.get_unchecked(j1).clone();
                    let y1 = y.get_unchecked(j2).clone();
                    
                    let w = w.get_unchecked(k);
                    let wy = w * y1;
                    *y.get_unchecked_mut(j1) = y0 + wy;
                    *y.get_unchecked_mut(j2) = y0 - wy;
                }
            }
        }
        
        hs = s;
        s <<= 1;
        p += 1;
    }
}
/// `y.len()`must be a power of 2
pub fn fft_iterative_v3<T: Float + ConstOne + ConstZero + FloatConst>(
    wn: &WCache<T>, y: &mut [Complex<T>])
{
    let n = y.len();
    let mut old = 0;
    let mut current = 2;
    while current <= n
    {
        let x = current & (!old);
        let mut hs = 1;
        let mut s = 2;
        let mut p = 1;
        while s <= x
        {
            unsafe
            {
                let w = wn.get_nth_roots_unchecked(p);
            
                let i = current - s;
                for k in 0..hs
                {
                    let j1 = k + i;
                    let j2 = j1 + hs;
                    let y0 = y.get_unchecked(j1).clone();
                    let y1 = y.get_unchecked(j2).clone();
                    
                    let w = w.get_unchecked(k);
                    let wy = w * y1;
                    *y.get_unchecked_mut(j1) = y0 + wy;
                    *y.get_unchecked_mut(j2) = y0 - wy;
                }
            }
            hs = s;
            s <<= 1;
            // s *= 2;
            p += 1;
        }
        
        old = current;
        current += 2;
    }
}