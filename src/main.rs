mod plot_element;
mod spectrum_element;
mod line_renderer;

use backend::{compute_nth_roots, dft_analysis, next_power_of_2};
use iced::{widget::{button, column, shader, slider, text}, Alignment, Element, Length, Padding};
use num::complex::Complex32;
use plot_element::{Plot, PlotData};
use spectrum_element::Spectrum;

pub const PLOTTER_SIZE: u32 = 200;

#[derive(Debug, Clone)]
enum Message
{
    Increment,
    Set(u32),
    PlotSize(usize),
    PlotPoint(usize, f32),
    PlotLine(usize, f32)
}

#[derive(Debug, Clone, Default)]
struct State
{
    counter: u32,
    plot: PlotData,
    spectrum: Vec<Complex32>,
    wn: Box<[Complex32]>,
    last_point: (usize, f32)
}

fn update_data(state: &mut State)
{
    let len = state.plot.points.len();
    if len == 0 { return; }
    
    let s = next_power_of_2(len);
    if state.wn.len() != s
    {
        state.wn = compute_nth_roots(s);
    }
    
    state.spectrum = dft_analysis(&state.wn, &state.plot.points);
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
    }
}

fn view(state: &State) -> Element<Message>
{
    let spec_size = state.spectrum.len().min(PLOTTER_SIZE as usize);
    column![
        text(state.counter).size(20),
        button("Increment").on_press(Message::Increment),
        slider(0..=50, state.counter, Message::Set),
        shader(Plot::new(Message::PlotSize, Message::PlotPoint, Message::PlotLine, &state.plot)).width(Length::Fixed(PLOTTER_SIZE as f32)),
        shader(Spectrum::new(&state.spectrum[0..spec_size])).width(Length::Fixed(PLOTTER_SIZE as f32))
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(Padding::new(5.0))
    .into()
}

fn main() {
    let _ = iced::run("Plotter", update, view);
}