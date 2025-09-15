mod plot;
mod screen;
mod wave_data;
mod scene;

use std::f32::consts::{PI, TAU};

use backend::{Colour, UIWall, WCache};
use iced::keyboard::Modifiers;
use iced::widget::{container, horizontal_rule};
use iced::{widget::{button, column, container::Style, row, slider, text, toggler, vertical_slider, Space}, Alignment, Background, Color, Element, Length, Padding};
use num::{complex::Complex32, Zero};
use plot::element::plotter;
use scene::element::MessageFuncs;
use scene::{Scene, SceneSlit, SceneUIData, SceneUIRef, DEFAULT_WIDTH};
use screen::element::screen;
use scene::element::scene;
use screen::renderer::SCREEN_SIZE;
use wave_data::WaveData;
use zene_structs::{Vector2, Vector4};

pub const PLOTTER_SIZE: u32 = 200;
pub const SPECTRUM_SIZE: u32 = 256;
pub const SL: f32 = 0.03;

const SCENE_MESSAGES: MessageFuncs<Message> = MessageFuncs
{
    on_zoom: Message::ZoomScene,
    on_pan: Message::PanScene,
    on_hover: Message::SceneHover,
    on_select: Message::SceneSelect,
    on_delete: Message::SceneDelete,
    on_cancel: Message::SceneCancel,
    on_drag: Message::SceneDrag,
    on_ghost: Message::GhostScene,
    on_ghost_end: Message::EndGhostScene,
};

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
    
    ZoomScene(f32, Vector2),
    PanScene(Vector2),
    SceneHover(SceneUIRef),
    SceneSelect(SceneUIRef),
    SceneDrag(SceneUIRef, Vector2<f64>, Vector2<f64>, Modifiers),
    SceneDelete(SceneUIRef),
    SceneCancel(),
    GhostScene(usize, f64),
    EndGhostScene(bool)
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
    scene_ui: SceneUIData,
    scene_ref_pos: Vector2<f64>,
    select_wall_old: (Vector2<f64>, Vector2<f64>),
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
            scene_ui,
            scene_ref_pos: Default::default(),
            select_wall_old: Default::default()
        }
    }
}

impl State
{
    fn drag_scene(&mut self, scene_uiref: SceneUIRef, pp: Vector2<f64>, wp: Vector2<f64>, mods: Modifiers)
    {
        match scene_uiref
        {
            SceneUIRef::Slit(i, j) =>
            {
                self.scene.set_slit_pos(i, j, wp);
            },
            SceneUIRef::Wall(i) =>// if mods.alt() =>
            {
                let a = self.scene_ref_pos + wp - pp;
                self.scene.get_ui_wall(i).shift_whole_wall(a);
            },
            SceneUIRef::Point(i, ab) =>
            {
                match (mods.shift(), mods.control())
                {
                    (true, true) => self.scene.get_ui_wall(i).snap_wall_points(ab, wp, self.select_wall_old, mods.alt()),
                    (false, true) => self.scene.get_ui_wall(i).set_wall_points(ab, wp, self.select_wall_old, mods.alt()),
                    (true, false) => self.scene.get_ui_wall(i).snap_wall_point(ab, wp, self.select_wall_old, mods.alt()),
                    (false, false) => self.scene.get_ui_wall(i).set_wall_point(ab, wp, self.select_wall_old, mods.alt())
                }
            },
            SceneUIRef::ScreenPoint(lr) =>
            {
                match (mods.shift(), mods.control())
                {
                    (true, true) => self.scene.env.screen.snap_wall_points(lr, wp, self.select_wall_old, mods.alt()),
                    (false, true) => self.scene.env.screen.set_wall_points(lr, wp, self.select_wall_old, mods.alt()),
                    (true, false) => self.scene.env.screen.snap_wall_point(lr, wp, self.select_wall_old, mods.alt()),
                    (false, false) => self.scene.env.screen.set_wall_point(lr, wp, self.select_wall_old, mods.alt())
                }
            },
            SceneUIRef::Screen =>// if mods.alt() =>
            {
                let a = self.scene_ref_pos + wp - pp;
                self.scene.env.screen.shift_whole_wall(a);
            },
            _ => return,
        }
        
        self.scene.simulate(&self.plot.wave_map, &mut self.colours);
        self.scene_ui.generate_lines(&self.scene, SL);
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
        Message::SceneHover(scene_uiref) =>
        {
            state.scene_ui.hover = scene_uiref;
            state.scene_ui.generate_lines(&state.scene, SL);
        },
        Message::SceneSelect(scene_uiref) =>
        {
            state.scene_ref_pos = state.scene.get_ref_pos(scene_uiref);
            if let SceneUIRef::Point(i, _) = scene_uiref
            {
                state.select_wall_old = state.scene.get_wall(i).get_a_b();
            }
            else if let SceneUIRef::ScreenPoint(_) = scene_uiref
            {
                state.select_wall_old = state.scene.env.screen;
            }
            
            if state.scene_ui.selection == scene_uiref { return; }
            state.scene_ui.selection = scene_uiref;
            state.scene_ui.generate_lines(&state.scene, SL);
        },
        Message::SceneDelete(scene_uiref) =>
        {
            match scene_uiref
            {
                SceneUIRef::None => return,
                SceneUIRef::Slit(wall, slit) => state.scene.delete_slit(wall, slit),
                SceneUIRef::Wall(wall) => state.scene.delete_wall(wall),
                SceneUIRef::Point(_, _) => return,
                SceneUIRef::ScreenPoint(_) => return,
                SceneUIRef::Screen => return,
            }
            
            if state.scene_ui.selection == scene_uiref
            {
                state.scene_ui.selection = SceneUIRef::None;
            }
            if state.scene_ui.hover == scene_uiref
            {
                state.scene_ui.hover = SceneUIRef::None;
            }
            
            state.scene_ui.generate_lines(&state.scene, SL);
        },
        Message::SceneDrag(scene_uiref, pp, wp, mods) =>
        {
            state.drag_scene(scene_uiref, pp, wp, mods);
        }
        Message::GhostScene(i, p) =>
        {
            let p = p.clamp(DEFAULT_WIDTH * 0.5, state.scene.get_wall(i).len() - (DEFAULT_WIDTH * 0.5));
            let ghost = (SceneSlit { width: DEFAULT_WIDTH, position: p }, i);
            state.scene_ui.ghost = Some(ghost);
            
            state.scene.simulate_ghost(&state.plot.wave_map, &mut state.colours, ghost);
            state.scene_ui.generate_lines(&state.scene, SL);
        },
        Message::EndGhostScene(valid) =>
        {
            if valid
            {
                // will exist
                let ghost = state.scene_ui.ghost.unwrap();
                state.scene.insert_slit(ghost);
            }
            
            state.scene_ui.ghost = None;
            state.scene.simulate(&state.plot.wave_map, &mut state.colours);
            state.scene_ui.generate_lines(&state.scene, SL);
        },
        Message::SceneCancel() =>
        {
            let old = state.scene_ref_pos;
            state.drag_scene(state.scene_ui.selection, old, old, Modifiers::empty());
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
            container(scene(&state.scene_ui.lines, &state.scene, state.scene_ui.zoom, state.scene_ui.pan, &SCENE_MESSAGES)
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