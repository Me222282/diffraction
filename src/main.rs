mod plot_element;
mod plot_renderer;
mod screen_element;
mod screen_renderer;
mod wave_data;
mod scene;
mod scene_element;
mod scene_renderer;

use std::f32::consts::{PI, TAU};

use backend::{Colour, WCache};
use iced::widget::{container, horizontal_rule};
use iced::{widget::{button, column, container::Style, row, slider, text, toggler, vertical_slider, Space}, Alignment, Background, Color, Element, Length, Padding};
use num::{complex::Complex32, Zero};
use plot_element::plotter;
use scene::{Scene, SceneUIData};
use screen_element::screen;
use scene_element::scene;
use screen_renderer::SCREEN_SIZE;
use wave_data::WaveData;
use zene_structs::{Vector2, Vector4};

pub const PLOTTER_SIZE: u32 = 200;
pub const SPECTRUM_SIZE: u32 = 256;
pub const SL: f32 = 0.05;

#[derive(Debug, Clone)]
enum Message
{
    SetScale(f32),
    SetExpo(f32),
    
    PlotSize(usize),
    PlotWave(usize, f32),
    DragWave(usize, f32),
    PlotFreq(usize, f32),
    DragFreq(usize, f32),
    PlotPhase(usize, f32),
    DragPhase(usize, f32),
    
    ViewPhase(bool),
    FillSine,
    FillTriangle,
    FillSaw,
    FillSquare,
    Clear,
    
    ZoomScene(f32, Vector2<f32>),
    PanScene(Vector2<f32>)
}

#[derive(Debug, Clone)]
struct State
{
    plot: WaveData,
    wn: WCache<f32>,
    last_point: (usize, f32),
    view_phase: bool,
    colours: Box<[Colour]>,
    exposure: f32,
    scene: Scene,
    scene_ui: SceneUIData
}
impl Default for State
{
    fn default() -> Self
    {
        let mut plot = WaveData::default();
        plot.set_scale(1.0);
        
        let scene = Scene::default();
        let scene_ui = SceneUIData::new(&scene, SL, 2.5e-10, Vector2::zero());
        return Self {
            view_phase: false,
            plot,
            wn: WCache::<f32>::new(true),
            last_point: Default::default(),
            colours: vec![Colour::ZERO; SCREEN_SIZE as usize].into_boxed_slice(),
            exposure: 1.0,
            scene,
            scene_ui
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
        Message::SetExpo(v) => state.exposure = v,
        Message::ViewPhase(v) =>
        {
            state.view_phase = v;
            state.plot.set_phase(v);
        }
        Message::PlotSize(size) =>
        {
            state.plot.resize(size);
            state.plot.compute_dft(&mut state.wn);
            
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::PlotWave(i, v) =>
        {
            state.plot.set_plot_point(i, v);
            state.plot.compute_dft(&mut state.wn);
            state.last_point = (i, v);
            
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::DragWave(i, v) =>
        {
            state.plot.set_plot_line(state.last_point, (i, v));
            state.plot.compute_dft(&mut state.wn);
            state.last_point = (i, v);
            
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::PlotFreq(i, v) =>
        {
            state.plot.set_spec_point(i, v);
            state.plot.compute_plot(&mut state.wn);
            state.last_point = (i, v);
            
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::DragFreq(i, v) =>
        {
            state.plot.set_spec_line(state.last_point, (i, v));
            state.plot.compute_plot(&mut state.wn);
            state.last_point = (i, v);
            
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::PlotPhase(i, v) =>
        {
            state.plot.set_phase_point(i, v);
            state.plot.compute_plot(&mut state.wn);
            state.last_point = (i, v);
            
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::DragPhase(i, v) =>
        {
            state.plot.set_phase_line(state.last_point, (i, v));
            state.plot.compute_plot(&mut state.wn);
            state.last_point = (i, v);
            
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        },
        Message::Clear =>
        {
            state.plot.wave.fill(0.0);
            state.plot.dft.fill(Complex32::ZERO);
            state.plot.update_spec_phase();
            
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
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
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
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
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
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
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
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
            state.scene.compute_waves(&state.plot);
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
        }
        Message::ZoomScene(zoom, pan) =>
        {
            state.scene_ui.zoom = zoom;
            state.scene_ui.pan = pan;
            
            state.scene_ui.generate_lines(&state.scene, SL);
        },
        Message::PanScene(pan) =>
        {
            state.scene_ui.pan = pan;
        }
    }
}

fn view(state: &State) -> Element<Message>
{
    let plot = &state.plot;
    let phase_el: Element<Message> = if state.view_phase
    {
        plotter::<fn(usize) -> Message, _, _, _, _, 2>(None, Message::PlotPhase, Message::DragPhase,
            &plot.phase, -PI..PI, Vector4::new(0.0, 1.0, 1.0, 1.0))
            .width(Length::Fixed(SPECTRUM_SIZE as f32)).into()
    }
    else
    {
        Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into()
    };
    
    let spec_scale = state.plot.get_scale();
    let view = row![
        column![
            screen(&state.colours, state.exposure),
            row![
                text(format!("Exposure: {:.3}", state.exposure)),
                slider(0.1..=10.0, state.exposure, Message::SetExpo).step(0.001)
                    .width(Length::Fill)
            ].spacing(10).width(Length::Fixed(SCREEN_SIZE as f32))
                .align_y(Alignment::Center)
                .padding(Padding::new(5.0)),
            container(scene(&state.scene_ui.lines, state.scene_ui.zoom, state.scene_ui.pan, Message::ZoomScene, Message::PanScene)
                .width(Length::Fill).height(Length::Fill)).center(Length::Fill)
                .style(|_| Style::default().background(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))))  
        ].spacing(10)
            .align_x(Alignment::Center)
            .padding(Padding::new(5.0)),
        column![
            row![
                button("Sine").on_press(Message::FillSine),
                button("Triangle").on_press(Message::FillTriangle),
                button("Saw").on_press(Message::FillSaw),
                button("Square").on_press(Message::FillSquare),
                button("Clear").on_press(Message::Clear),
                toggler(state.view_phase)
                    .label("Phase")
                    .on_toggle(Message::ViewPhase)
            ].spacing(10)
                .align_y(Alignment::Center)
                .padding(Padding::new(5.0)),
                
            plotter::<_, _, _, _, _, 0>(Some(Message::PlotSize), Message::PlotWave, Message::DragWave,
                &plot.wave, -1.0..1.0, Vector4::new(1.0, 0.0, 0.0, 1.0))
                .width(Length::Fixed(PLOTTER_SIZE as f32)),
            
            row![
                plotter::<fn(usize) -> Message, _, _, _, _, 1>(None, Message::PlotFreq, Message::DragFreq,
                    &plot.spectrum, 0.0..1.0, Vector4::zero())
                    .width(Length::Fixed(SPECTRUM_SIZE as f32)),
                
                column![
                    vertical_slider(1.0..=5.0, spec_scale, Message::SetScale).step(0.01)
                        .height(Length::Fill),
                    text(format!("{spec_scale:.2}"))
                ].align_x(Alignment::Center).height(Length::Fill)
            ].spacing(10).height(Length::Shrink)
                .align_y(Alignment::Center)
                .padding(Padding::new(5.0)),
            
            phase_el,
            horizontal_rule(2)
        ].spacing(10).width(Length::Shrink)
            .align_x(Alignment::Center)
            .padding(Padding::new(5.0))
    ].spacing(10)
    .align_y(Alignment::Start)
    .padding(Padding::new(5.0));
    
    return view.into();
}

fn main() {
    let _ = iced::application("Plotter", update, view)
        .theme(|_| iced::Theme::Dark)
        .run();
}