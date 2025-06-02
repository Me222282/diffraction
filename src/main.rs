mod plot_element;
mod spectrum_element;
mod plot_renderer;
mod spectrum_renderer;

use std::f32::consts::TAU;

use backend::{dft_analysis, next_power_of_2, RepeatUntil, WCache};
use iced::{widget::{button, column, shader, slider, text}, Alignment, Element, Length, Padding};
use num::{complex::Complex32, Complex};
use plot_element::{Plot, PlotData};
use rustfft::Fft;
use spectrum_element::Spectrum;

pub const PLOTTER_SIZE: u32 = 200;
pub const SPECTRUM_SIZE: u32 = 256;

#[derive(Debug, Clone)]
enum Message
{
    Increment,
    Set(u32),
    PlotSize(usize),
    PlotPoint(usize, f32),
    PlotLine(usize, f32),
    FillSine
}

#[derive(Debug, Clone, Default)]
struct State
{
    counter: u32,
    plot: PlotData,
    spectrum: Vec<Complex32>,
    wn: WCache<f32>,
    last_point: (usize, f32)
}

fn update_data(state: &mut State)
{
    let len = state.plot.points.len();
    if len == 0 { return; }
    
    // check means repeated calls do nothing
    state.wn.set_invert(true);
    state.spectrum = dft_analysis(&mut state.wn, &state.plot.points);
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
fn fill(plot: &mut PlotData, start: (usize, f32), end: (usize, f32))
{
    if start.0 > end.0
    {
        fill(plot, end, start);
        return;
    }
    
    let diff = end.1 - start.1;
    let scale = 1.0 / (1 + end.0 - start.0) as f32;
    
    // first is already done
    for p in plot.points[start.0..=end.0].iter_mut().enumerate().skip(1)
    {
        let v = p.0 as f32 * scale;
        *p.1 = start.1 + (diff * v);
    }
}

fn update(state: &mut State, message: Message)
{
    match message
    {
        Message::Increment => state.counter += 1,
        Message::Set(v) => state.counter = v,
        Message::PlotSize(size) =>
        {
            let mut new = vec![0.0; size];
            
            remap(&state.plot.points, &mut new);
            
            state.plot.points = new;
            update_data(state);
        },
        Message::PlotPoint(i, v) =>
        {
            state.plot.points[i] = v;
            state.last_point = (i, v);
            update_data(state);
        },
        Message::PlotLine(i, v) =>
        {
            fill(&mut state.plot, state.last_point, (i, v));
            state.last_point = (i, v);
            update_data(state);
        }
        Message::FillSine =>
        {
            let step = TAU / (state.plot.points.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.points.iter_mut()
            {
                *v = t.sin();
                t += step;
            }
            update_data(state);
        },
    }
}

fn view(state: &State) -> Element<Message>
{
    let spec_size = state.spectrum.len().min(SPECTRUM_SIZE as usize);
    column![
        text(state.counter).size(20),
        button("Increment").on_press(Message::Increment),
        slider(0..=50, state.counter, Message::Set),
        button("Sine").on_press(Message::FillSine),
        shader(Plot::new(Message::PlotSize, Message::PlotPoint, Message::PlotLine, &state.plot)).width(Length::Fixed(PLOTTER_SIZE as f32)),
        shader(Spectrum::new(&state.spectrum[0..spec_size])).width(Length::Fixed(SPECTRUM_SIZE as f32))
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(Padding::new(5.0))
    .into()
}

fn main() {
    let _ = iced::run("Plotter", update, view);
}