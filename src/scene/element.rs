use iced::event::Status;
use iced::mouse::{Button, ScrollDelta};
use iced::widget::shader::Event;
use iced::widget::{shader::Program, shader};
use iced::widget::Shader;
use iced::{Point, Rectangle};
use zene_structs::Vector2;

use super::{LineData, Scene, SceneUIRef};
use super::renderer::SceneRender;

pub struct MessageFuncs<Message>
{
    pub on_zoom: fn(f32, Vector2) -> Message,
    pub on_pan: fn(Vector2) -> Message,
    pub on_hover: fn(SceneUIRef) -> Message,
    pub on_select: fn(SceneUIRef) -> Message,
    // pub on_drag: fn(SceneUIRef, Vector2) -> Message
}

pub fn scene<'a, Message: 'static>(lines: &'a [LineData], scene: &'a Scene, zoom: f32, pan: Vector2,
    funcs: &'static MessageFuncs<Message>) -> Shader<Message, SceneEl<'a, Message>>
{
    return shader(
        SceneEl { lines, scene, zoom, pan, funcs }
    );
}

pub struct SceneEl<'a, Message: 'static>
{
    lines: &'a [LineData],
    scene: &'a Scene,
    zoom: f32,
    pan: Vector2,
    funcs: &'static MessageFuncs<Message>
}

#[derive(Debug, Clone, Default)]
pub struct State
{
    panning: bool,
    mp: Point,
    hover: SceneUIRef,
    select: SceneUIRef,
    press_select: bool
}

impl<'a, Message: 'static> Program<Message> for SceneEl<'a, Message>
{
    type State = State;
    type Primitive = SceneRender;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: Rectangle) -> Self::Primitive
    {
        return SceneRender::new(self.lines.to_vec(), self.zoom, self.pan);
    }
    
    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
        shell: &mut iced::advanced::Shell<'_, Message>) -> (Status, Option<Message>)
    {
        let centre = Vector2::new(bounds.width * 0.5, bounds.height * -0.5);
        let pan_div = if bounds.width < bounds.height
        {
            centre.x
        } else
        {
            -centre.y
        };
        
        match event
        {
            Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Middle)) =>
            {
                if let Some(cursor_position) = cursor.position_over(bounds)
                {
                    state.panning = true;
                    state.mp = cursor_position;
                    
                    if state.hover != SceneUIRef::None
                    {
                        state.hover = SceneUIRef::None;
                        shell.publish((self.funcs.on_hover)(SceneUIRef::None));
                    }
                    
                    return (Status::Captured, None);
                }
            },
            Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Middle)) =>
            {
                if state.panning
                {
                    state.panning = false;
                    
                    let mp = cursor.position_from(bounds.center()).unwrap_or(Default::default());
                    let mp = Vector2::new(mp.x, -mp.y);
                    let wp = ((mp / pan_div) - self.pan) / self.zoom;
                    
                    let hover = self.scene.mouse_point(Vector2::new(wp.x as f64, wp.y as f64), self.zoom);
                    
                    if hover != state.hover
                    {
                        state.hover = hover;
                        shell.publish((self.funcs.on_hover)(hover));
                    }
                    
                    return (Status::Captured, None);
                }
            },
            Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left)) =>
            {
                if cursor.is_over(bounds)
                {
                    match state.hover
                    {
                        SceneUIRef::Slit(_, _) =>
                        {
                            state.select = state.hover;
                            state.press_select = true;
                            shell.publish((self.funcs.on_hover)(SceneUIRef::None));
                        }
                        SceneUIRef::Wall(_) =>
                        {
                            state.select = state.hover;
                            state.press_select = true;
                            shell.publish((self.funcs.on_hover)(SceneUIRef::None));
                        },
                        _ => state.select = SceneUIRef::None
                    }
                    
                    shell.publish((self.funcs.on_select)(state.select));
                    
                    return (Status::Captured, None);
                }
            },
            Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left)) =>
            {
                if state.press_select
                {
                    shell.publish((self.funcs.on_hover)(state.hover));
                    
                    return (Status::Captured, None);
                }
            },
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) =>
            {
                let old_pos = state.mp;
                state.mp = position;
                
                if state.panning
                {
                    let diff = position - old_pos;
                    let np = self.pan + (Vector2::new(diff.x, -diff.y) / pan_div);
                    
                    return (Status::Captured, Some((self.funcs.on_pan)(np)));
                }
                
                let mp = cursor.position_from(bounds.center()).unwrap_or(Default::default());
                let mp = Vector2::new(mp.x, -mp.y);
                let wp = ((mp / pan_div) - self.pan) / self.zoom;
                
                let hover = self.scene.mouse_point(Vector2::new(wp.x as f64, wp.y as f64), self.zoom);
                
                if hover != state.hover
                {
                    state.hover = hover;
                    shell.publish((self.funcs.on_hover)(hover));
                }
            },
            Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) =>
            {
                if state.panning || cursor.is_over(bounds)
                {
                    match delta
                    {
                        ScrollDelta::Lines { x: _, y } =>
                        'new_zoom: {
                            let nz = self.zoom + (y * 0.1 * self.zoom);

                            if nz < 0.0 { break 'new_zoom; }
                            
                            let mp = cursor.position_from(bounds.center()).unwrap_or(Default::default());
                            
                            let pan = self.pan * pan_div;
                            let mp = Vector2::new(mp.x, -mp.y);
                            
                            let point_rel_old = (mp - pan) / self.zoom;
                            let point_rel_new = (mp - pan) / nz;

                            let np = pan + (point_rel_new - point_rel_old) * nz;
                            return (Status::Captured, Some((self.funcs.on_zoom)(nz, np / pan_div)));
                        }
                        _ => {}
                    }
                }
            },
            _ => {}
        }
        
        return (Status::Ignored, None);
    }
    
    // fn mouse_interaction(
    //     &self,
    //     state: &Self::State,
    //     bounds: iced::Rectangle,
    //     cursor: iced::advanced::mouse::Cursor) -> iced::advanced::mouse::Interaction
    // {
    //     let is_mouse_over = cursor.is_over(bounds);
        
    //     if *state
    //     {
    //         iced::advanced::mouse::Interaction::Pointer
    //     }
    //     else if is_mouse_over
    //     {
    //         iced::advanced::mouse::Interaction::Pointer
    //     }
    //     else
    //     {
    //         iced::advanced::mouse::Interaction::default()
    //     }
    // }
}