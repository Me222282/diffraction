mod plot_element;
mod spectrum_element;
mod plot_renderer;
mod spectrum_renderer;

use std::f32::consts::TAU;

use backend::{dft_analysis, form_plot, WCache};
use iced::{widget::{button, column, row, shader, slider, text}, Alignment, Element, Length, Padding};
use num::{complex::Complex32, NumCast};
use plot_element::{Plot, PlotData};
use spectrum_element::Spectrum;
use zene_structs::{ConstOne, NumOps};

pub const PLOTTER_SIZE: u32 = 200;
pub const SPECTRUM_SIZE: u32 = 256;

#[derive(Debug, Clone)]
enum Message
{
    SetScale(f32),
    PlotSize(usize),
    PlotPoint(usize, f32),
    PlotLine(usize, f32),
    PlotFreq(f32, f32),
    DragFreq(f32, f32),
    FillSine,
    FillTriangle,
    FillSaw,
    FillSquare,
    Clear
}

#[derive(Debug, Clone)]
struct State
{
    spec_scale: f32,
    plot: PlotData,
    spectrum: Vec<Complex32>,
    wn: WCache<f32>,
    wn_back: WCache<f32>,
    last_point: (usize, f32)
}
impl Default for State
{
    fn default() -> Self
    {
        return Self {
            spec_scale: 1.0,
            plot: Default::default(),
            spectrum: Default::default(),
            wn: WCache::<f32>::new(true),
            wn_back: WCache::<f32>::new(false),
            last_point: Default::default()
        }
    }
}

fn update_spectrum(state: &mut State)
{
    let len = state.plot.points.len();
    if len == 0 { return; }
    
    state.spectrum = dft_analysis(&mut state.wn, &state.plot.points);
}
fn update_plot(state: &mut State)
{
    let len = state.plot.points.len();
    if len == 0 { return; }
    
    let plot = form_plot(&mut state.wn_back, &state.spectrum, state.plot.points.len());
    state.plot.points = plot.iter().map(|c| c.re).collect();
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
fn fill<F>(plot: &mut [F], start: (usize, F), end: (usize, F))
    where F: NumOps + ConstOne + NumCast + Copy
{
    if start.0 > end.0
    {
        fill(plot, end, start);
        return;
    }
    
    let diff = end.1 - start.1;
    let scale = F::ONE / F::from(1 + end.0 - start.0).unwrap();
    
    // first is already done
    for p in plot[start.0..=end.0].iter_mut().enumerate().skip(1)
    {
        let v = scale * F::from(p.0).unwrap();
        *p.1 = start.1 + (diff * v);
    }
}

fn tri(p: f32) -> f32
{
    return ((p + 0.25 - (p + 0.75).floor()).abs() * 4.0) - 1.0;
}
fn saw(p: f32) -> f32
{
    return ((p - p.floor()) * 2.0) - 1.0;
}
fn square(p: f32) -> f32
{
    return ((((p * 2.0) as isize % 2) * 2) - 1) as f32;
}

fn update(state: &mut State, message: Message)
{
    match message
    {
        Message::SetScale(v) => state.spec_scale = v,
        Message::PlotSize(size) =>
        {
            let mut new = vec![0.0; size];
        
            remap(&state.plot.points, &mut new);
        
            state.plot.points = new;
            update_spectrum(state);
        },
        Message::PlotPoint(i, v) =>
        {
            state.plot.points[i] = v;
            state.last_point = (i, v);
            update_spectrum(state);
        },
        Message::PlotLine(i, v) =>
        {
            fill(&mut state.plot.points, state.last_point, (i, v));
            state.last_point = (i, v);
            update_spectrum(state);
        }
        Message::FillSine =>
        {
            let step =  TAU / (state.plot.points.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.points.iter_mut()
            {
                *v = t.sin();
                t += step;
            }
            update_spectrum(state);
        },
        Message::FillTriangle =>
        {
            let step =  1.0 / (state.plot.points.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.points.iter_mut()
            {
                *v = tri(t);
                t += step;
            }
            update_spectrum(state);
        },
        Message::FillSaw =>
        {
            let step =  1.0 / (state.plot.points.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.points.iter_mut()
            {
                *v = saw(1.0 - t);
                t += step;
            }
            update_spectrum(state);
        },
        Message::FillSquare =>
        {
            let step =  1.0 / (state.plot.points.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.points.iter_mut()
            {
                *v = square(1.0 - t);
                t += step;
            }
            update_spectrum(state);
        },
        Message::PlotFreq(x, v) =>
        {
            let lf = state.spectrum.len() as f32;
            let v = v * lf / state.spec_scale;
            let i = (x * (lf - 2.0)) as usize + 1;
            state.spectrum[i] = Complex32::new(v, 0.0);
            state.last_point = (i, v);
            update_plot(state);
        },
        Message::DragFreq(x, v) =>
        {
            let lf = state.spectrum.len() as f32;
            let v = v * lf / state.spec_scale;
            let i = (x * (lf - 2.0)) as usize + 1;
            fill(&mut state.spectrum,
                (state.last_point.0, Complex32::new(state.last_point.1, 0.0)),
                (i, Complex32::new(v, 0.0)));
            state.last_point = (i, v);
            update_plot(state);
        }
        Message::Clear =>
        {
            state.plot.points.fill(0.0);
            state.spectrum.fill(Complex32::ZERO);
        },
    }
}

fn view(state: &State) -> Element<Message>
{
    let spec_size = state.spectrum.len().min(SPECTRUM_SIZE as usize);
    column![
        row![
            button("Sine").on_press(Message::FillSine),
            button("Triangle").on_press(Message::FillTriangle),
            button("Saw").on_press(Message::FillSaw),
            button("Square").on_press(Message::FillSquare),
            button("Clear").on_press(Message::Clear)
        ].spacing(10)
            .align_y(Alignment::Center)
            .padding(Padding::new(5.0)),
        shader(Plot::new(Message::PlotSize, Message::PlotPoint, Message::PlotLine, &state.plot))
            .width(Length::Fixed(PLOTTER_SIZE as f32)),
        shader(Spectrum::new(Message::PlotFreq, Message::DragFreq, &state.spectrum[0..spec_size], state.spec_scale))
            .width(Length::Fixed(SPECTRUM_SIZE as f32)),
        slider(1.0..=5.0, state.spec_scale, Message::SetScale).step(0.01)
            .width(Length::Fixed(SPECTRUM_SIZE as f32)),
        text(format!("Spectrum Scale: {:.2}", state.spec_scale))
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(Padding::new(5.0))
    .into()
}

fn main() {
    let _ = iced::run("Plotter", update, view);
}