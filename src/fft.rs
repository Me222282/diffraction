use num::{traits::{ConstOne, ConstZero, FloatConst}, Complex, Float};

pub fn compute_nth_roots<T: Float + ConstOne + ConstZero + FloatConst>(n: usize) -> Vec<Complex<T>>
{
    let mut result = Vec::<Complex<T>>::with_capacity(n);
    let mut w = Complex::<T>::ONE;
    let wn = Complex::cis(T::TAU() / T::from(n).unwrap());
    result[0] = w;
    
    for i in 1..n
    {
        w = w * wn;
        result[i] = w;
    }
    
    return result;
}