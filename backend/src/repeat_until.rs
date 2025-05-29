#[derive(Clone)]
pub struct RepeatUntil<'a, T>
{
    slice: &'a [T],
    limit: usize,
    count: usize,
    sub_count: usize
}

impl<'a, T> RepeatUntil<'a, T>
{
    pub fn new(slice: &'a [T], limit: usize) -> Self
    {
        return Self { slice, limit, count: 0, sub_count: 0 };
    }
}

impl<'a, T> Iterator for RepeatUntil<'a, T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T>
    {
        if self.count >= self.limit
        {
            return None;
        }
        self.count += 1;
        
        let len = self.slice.len();
        let mut sc = self.sub_count;
        if sc >= len
        {
            sc -= len;
        }
        
        let v;
        unsafe
        {
            v = self.slice.get_unchecked(sc);
        }
        sc += 1;
        self.sub_count = sc;
        return Some(v);
    }
    
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let s = self.limit - self.count;
        return (s, Some(s));
    }
    
    fn count(self) -> usize
    where
        Self: Sized,
    {
        return self.limit - self.count;
    }
}