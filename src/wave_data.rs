use backend::{dft_analysis, form_plot, wave_length_colour, WCache};
use num::{complex::Complex32, NumCast, traits::{ConstOne, NumOps}};

pub fn fill<F>(plot: &mut [F], start: (usize, F), end: (usize, F))
    where F: NumOps + ConstOne + NumCast + Copy
{
    fill_format(plot, start, end, |o, i| *o = i);
}
pub fn fill_format<'a, I: 'a, F, G: 'a, W>(plot: &'a mut I, start: (usize, F), end: (usize, F), write: W)
    where F: NumOps + ConstOne + NumCast + Copy,
        W: Fn(G, F),
        I: ?Sized,
        &'a mut I: IntoIterator<Item = G>
{
    if start.0 == end.0 { return; }
    if start.0 > end.0
    {
        fill_format(plot, end, start, write);
        return;
    }

    let diff = end.1 - start.1;
    let scale = F::ONE / F::from(1 + end.0 - start.0).unwrap();
    
    for p in plot.into_iter().skip(start.0).take(end.0 - start.0 + 1).enumerate()
    {
        let v = scale * F::from(p.0).unwrap();
        write(p.1, start.1 + (diff * v));
    }
}

fn lerp_index(vec: &Vec<f32>, i: f32) -> f32
{
    let ld = i.floor();
    let l = ld as usize;
    let u = i.ceil() as usize;
    
    if u >= vec.len()
    {
        return vec[l];
    }
    let i = i - ld;
    
    let a = vec[l];
    let b = vec[u];
    return ((b - a) * i) + a;
}
fn remap(a: &Vec<f32>, b: &mut Vec<f32>)
{
    let al = a.len();
    // no data to remap
    if al == 0 { return; }
    
    let step = (al as f32) / (b.len() as f32);
    
    let mut fi = 0.0;
    for v in b.iter_mut()
    {
        *v = lerp_index(a, fi);
        fi += step;
    }
}

#[derive(Debug, Clone, Default)]
pub struct WaveData
{
    pub wave: Vec<f32>,
    pub spectrum: Vec<[f32; 4]>,
    pub phase: Vec<f32>,
    pub dft: Vec<Complex32>,
    scale: f32,
    use_phase: bool
}

impl WaveData
{
    pub fn compute_dft(&mut self, wn: &mut WCache<f32>)
    {
        if self.wave.len() == 0 { return; }
        
        self.dft = dft_analysis(wn, &self.wave);
        self.update_spec_phase();
    }
    pub fn compute_plot(&mut self, wn_back: &mut WCache<f32>)
    {
        if self.dft.len() == 0 { return; }
        
        let plot = form_plot(wn_back, &self.dft, self.wave.len());
        self.wave = plot.iter().map(|c| c.re).collect();
    }
    pub fn resize(&mut self, size: usize)
    {
        let mut new = vec![0.0; size];
        
        remap(&self.wave, &mut new);
    
        self.wave = new;
    }
    pub fn update_spec_phase(&mut self)
    {
        let s = self.scale / (self.dft.len() as f32);
        let t = 300.0 / (self.dft.len() as f32);
        self.spectrum = self.dft.iter().skip(1).enumerate().map(|p|
        {
            let v = p.0 as f32 * t;
            let c = wave_length_colour(700.0 - v, 0.8);
            let amp = p.1.norm();
            return [amp * s, c.x, c.y, c.z];
        }).collect();
        
        self.phase = self.dft.iter().skip(1).map(|c| c.arg()).collect();
    }
    pub fn set_scale(&mut self, scale: f32)
    {
        let s = scale / self.scale;
        self.scale = scale;
        if self.spectrum.len() == 0 { return; }
        
        for v in &mut self.spectrum
        {
            v[0] *= s;
        }
    }
    pub fn get_scale(&self) -> f32
    {
        return self.scale;
    }
    pub fn set_phase(&mut self, phase: bool)
    {
        self.use_phase = phase;
    }
    
    pub fn set_plot_point(&mut self, index: usize, value: f32)
    {
        self.wave[index] = value;
    }
    pub fn set_plot_line(&mut self, start: (usize, f32), end: (usize, f32))
    {
        fill(&mut self.wave, start, end);
    }
    pub fn set_spec_point(&mut self, index: usize, value: f32)
    {
        let old = self.spectrum[index][0];
        self.spectrum[index][0] = value;
        
        if !self.use_phase
        {
            self.dft[index + 1] = Complex32::new(value * (self.dft.len() as f32) / self.scale, 0.0);
            return;
        }
        
        if old == 0.0
        {
            let phase = self.phase[index];
            self.dft[index + 1] = Complex32::from_polar(
                value * (self.dft.len() as f32) / self.scale,
                phase);
            return;
        }
        
        // rescale value
        self.dft[index + 1] = self.dft[index + 1] * (value / old);
    }
    pub fn set_spec_line(&mut self, start: (usize, f32), end: (usize, f32))
    {
        let s = (self.dft.len() as f32) / self.scale;
        
        let mut iter = self.spectrum.iter_mut().zip(&self.phase).zip(self.dft.iter_mut().skip(1));
        if self.use_phase
        {
            fill_format(&mut iter, start, end, |out, amp|
            {
                let old = out.0.0[0];
                out.0.0[0] = amp;
                // compute new complex value
                match old
                {
                    0.0 => *out.1 = Complex32::from_polar(amp * s, *out.0.1),
                    // keep direction (phase)
                    _ => *out.1 = *out.1 * (amp / old)
                }
            });
            return;
        }
        
        fill_format(&mut iter, start, end, |out, amp|
        {
            out.0.0[0] = amp;
            *out.1 = Complex32::new(amp * s, 0.0);
        });
    }
    pub fn set_phase_point(&mut self, index: usize, value: f32)
    {
        self.phase[index] = value;
        
        let amp = self.spectrum[index][0] * (self.dft.len() as f32) / self.scale;
        // new phase
        self.dft[index + 1] = Complex32::from_polar(amp, value);
    }
    pub fn set_phase_line(&mut self, start: (usize, f32), end: (usize, f32))
    {
        let s = (self.dft.len() as f32) / self.scale;
        
        let mut iter = self.phase.iter_mut().zip(&self.spectrum).zip(self.dft.iter_mut().skip(1));
        fill_format(&mut iter, start, end, |o, new|
        {
            *o.0.0 = new;
            let amp = o.0.1[0] * s;
            // compute new complex
            *o.1 = Complex32::from_polar(amp, new);
        });
    }
}