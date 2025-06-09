mod plot_element;
mod line_renderer;
mod wave_data;

use std::f32::consts::{PI, TAU};

use backend::WCache;
use iced::{widget::{button, column, row, slider, text}, Alignment, Element, Length, Padding};
use num::{complex::Complex32, Zero};
use plot_element::plotter;
use wave_data::WaveData;
use zene_structs::Vector4;

pub const PLOTTER_SIZE: u32 = 200;
pub const SPECTRUM_SIZE: u32 = 256;

#[derive(Debug, Clone)]
enum Message
{
    SetScale(f32),
    PlotSize(usize),
    PlotWave(usize, f32),
    DragWave(usize, f32),
    PlotFreq(usize, f32),
    DragFreq(usize, f32),
    PlotPhase(usize, f32),
    DragPhase(usize, f32),
    
    FillSine,
    FillTriangle,
    FillSaw,
    FillSquare,
    Clear
}

#[derive(Debug, Clone)]
struct State
{
    plot: WaveData,
    wn: WCache<f32>,
    wn_back: WCache<f32>,
    last_point: (usize, f32)
}
impl Default for State
{
    fn default() -> Self
    {
        let mut plot = WaveData::default();
        plot.set_scale(1.0);
        
        return Self {
            plot,
            wn: WCache::<f32>::new(true),
            wn_back: WCache::<f32>::new(false),
            last_point: Default::default()
        }
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
        Message::SetScale(v) => state.plot.set_scale(v),
        Message::PlotSize(size) =>
        {
            state.plot.resize(size);
            state.plot.compute_dft(&mut state.wn);
        },
        Message::PlotWave(i, v) =>
        {
            state.plot.set_plot_point(i, v);
            state.plot.compute_dft(&mut state.wn);
            state.last_point = (i, v);
        },
        Message::DragWave(i, v) =>
        {
            state.plot.set_plot_line(state.last_point, (i, v));
            state.plot.compute_dft(&mut state.wn);
            state.last_point = (i, v);
        },
        Message::PlotFreq(i, v) =>
        {
            state.plot.set_spec_point(i, v);
            state.plot.compute_plot(&mut state.wn_back);
            state.last_point = (i, v);
        },
        Message::DragFreq(i, v) =>
        {
            state.plot.set_spec_line(state.last_point, (i, v));
            state.plot.compute_plot(&mut state.wn_back);
            state.last_point = (i, v);
        },
        Message::PlotPhase(i, v) =>
        {
            state.plot.set_phase_point(i, v);
            state.plot.compute_plot(&mut state.wn_back);
            state.last_point = (i, v);
        },
        Message::DragPhase(i, v) =>
        {
            state.plot.set_phase_line(state.last_point, (i, v));
            state.plot.compute_plot(&mut state.wn_back);
            state.last_point = (i, v);
        },
        Message::Clear =>
        {
            state.plot.wave.fill(0.0);
            state.plot.dft.fill(Complex32::ZERO);
            state.plot.update_spec_phase();
        },
        Message::FillSine =>
        {
            let step =  TAU / (state.plot.wave.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.wave.iter_mut()
            {
                *v = t.sin();
                t += step;
            }
            state.plot.compute_dft(&mut state.wn);
        },
        Message::FillTriangle =>
        {
            let step =  1.0 / (state.plot.wave.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.wave.iter_mut()
            {
                *v = tri(t);
                t += step;
            }
            state.plot.compute_dft(&mut state.wn);
        },
        Message::FillSaw =>
        {
            let step =  1.0 / (state.plot.wave.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.wave.iter_mut()
            {
                *v = saw(1.0 - t);
                t += step;
            }
            state.plot.compute_dft(&mut state.wn);
        },
        Message::FillSquare =>
        {
            let step =  1.0 / (state.plot.wave.len() as f32);
            let mut t = 0.0f32;
            for v in state.plot.wave.iter_mut()
            {
                *v = square(1.0 - t);
                t += step;
            }
            state.plot.compute_dft(&mut state.wn);
        }
    }
}

fn view(state: &State) -> Element<Message>
{
    let spec_scale = state.plot.get_scale();
    let plot = &state.plot;
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
            
        plotter::<_, _, _, _, _, 0>(Some(Message::PlotSize), Message::PlotWave, Message::DragWave,
            &plot.wave, -1.0..1.0, Vector4::new(1.0, 0.0, 0.0, 1.0))
            .width(Length::Fixed(PLOTTER_SIZE as f32)),
            
        plotter::<fn(usize) -> Message, _, _, _, _, 1>(None, Message::PlotFreq, Message::DragFreq,
            &plot.spectrum, 0.0..1.0, Vector4::zero())
            .width(Length::Fixed(SPECTRUM_SIZE as f32)),
            
        slider(1.0..=5.0, spec_scale, Message::SetScale).step(0.01)
            .width(Length::Fixed(SPECTRUM_SIZE as f32)),
        text(format!("Spectrum Scale: {spec_scale:.2}")),
        
        plotter::<fn(usize) -> Message, _, _, _, _, 1>(None, Message::PlotPhase, Message::DragPhase,
            &plot.phase, -PI..PI, Vector4::new(0.0, 1.0, 1.0, 1.0))
            .width(Length::Fixed(SPECTRUM_SIZE as f32))
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(Padding::new(5.0))
    .into()
}

fn main() {
    let _ = iced::run("Plotter", update, view);
}