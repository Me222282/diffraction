use std::sync::Arc;

use iced::mouse::Button;
use iced::widget::shader::{Event, Program};
use iced::advanced::graphics::core::event::Status;
use iced::{Point, Rectangle};
use zene_structs::Vector4;

use crate::line_renderer::Lines;

pub struct PlotData
{
    pub points: Arc<[f32]>
}

#[derive(Default)]
pub struct PlotState
{
    mouse_hold: bool
}

pub struct Plot<'a, S, F, Message>
    where S: Fn(usize) -> Message,
        F: Fn(usize, f32) -> Message
{
    on_size: S,
    on_value: F,
    data: &'a PlotData,
    colour: Vector4<f32>
}

impl<'a, S, F, Message> Program<Message> for Plot<'a, S, F, Message>
    where S: Fn(usize) -> Message,
        F: Fn(usize, f32) -> Message
{
    type State = PlotState;
    type Primitive = Lines;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        bounds: Rectangle) -> Self::Primitive
    {
        return Lines::new(self.colour, self.data.points.clone(), bounds);
    }
    
    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
        shell: &mut iced::advanced::Shell<'_, Message>) -> (Status, Option<Message>)
    {
        let width = bounds.width as usize;
        
        if self.data.points.len() != width
        {
            shell.publish((self.on_size)(width));
        }
        
        match event
        {
            Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left)) =>
            {
                if let Some(cursor_position) = cursor.position_over(bounds)
                {
                    state.mouse_hold = true;
                    let p = cursor_position - bounds.position();
                    let v = (bounds.height - p.y) / bounds.height;
                    let x = p.x as usize;
                    
                    return (Status::Captured, Some((self.on_value)(x, v)));
                }
            },
            Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left)) =>
            {
                if cursor.position_over(bounds).is_some()
                {
                    state.mouse_hold = false;
                    return (Status::Captured, None);
                }
            },
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) =>
            {
                if state.mouse_hold
                {
                    let p = position - bounds.position();
                    let p = Point::new(p.x.clamp(0.0, bounds.width), p.y.clamp(0.0, bounds.height));
                    let v = (bounds.height - p.y) / bounds.height;
                    let x = p.x as usize;
                    
                    return (Status::Captured, Some((self.on_value)(x, v)));
                }
            },
            _ => {}
        }
        
        return (Status::Ignored, None);
    }
    
    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor) -> iced::advanced::mouse::Interaction
    {
        let is_mouse_over = cursor.is_over(bounds);
        
        if state.mouse_hold
        {
            iced::advanced::mouse::Interaction::Pointer
        }
        else if is_mouse_over
        {
            iced::advanced::mouse::Interaction::Pointer
        }
        else
        {
            iced::advanced::mouse::Interaction::default()
        }
    }
}