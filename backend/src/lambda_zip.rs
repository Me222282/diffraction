use num::Float;

use crate::Wave;

#[derive(Clone)]
pub struct LambdaZip<'a, T: Float + 'a, A: 'a, I1, I2>
    where I1: Iterator<Item = &'a mut (T, A)>,
        I2: Iterator<Item = &'a Wave<T>>
{
    left: I1,
    right: I2
}

impl<'a, T: Float + 'a, A: 'a, I1, I2> LambdaZip<'a, T, A, I1, I2>
    where I1: Iterator<Item = &'a mut (T, A)>,
        I2: Iterator<Item = &'a Wave<T>>
{
    pub fn new(left: I1, right: I2) -> Self
    {
        return Self { left, right };
    }
}

impl<'a, T: Float + 'a, A: 'a, I1, I2> Iterator for LambdaZip<'a, T, A, I1, I2>
    where I1: Iterator<Item = &'a mut (T, A)>,
        I2: Iterator<Item = &'a Wave<T>>,
        T: PartialEq
{
    type Item = (&'a mut A, &'a Wave<T>);

    fn next(&mut self) -> Option<Self::Item>
    {
        let b = self.right.next();
        
        return match b
        {
            Some(t) =>
            {
                // find next left that matches right, or none
                loop
                {
                    let x = self.left.next();
                    if let Some(v) = x
                    {
                        if v.0 != t.lambda { continue; }
                        
                        return Some((&mut v.1, t));
                    }
                    
                    return None;
                }
            },
            None => None
        };
    }
    
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let l = self.left.size_hint();
        let r = self.right.size_hint();
        return (l.0.min(r.0), l.1.map_or(r.1, |lv| r.1.map_or(Some(lv), |rv| Some(lv.min(rv)))));
    }
}