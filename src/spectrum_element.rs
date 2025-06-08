use backend::wave_length_colour;
use iced::mouse::Button;
use iced::widget::shader::{Event, Program};
use iced::advanced::graphics::core::event::Status;
use iced::{Point, Rectangle};
use num::complex::Complex32;

use crate::spectrum_renderer::Lines;

pub struct Spectrum<'a, F, G, Message>
    where F: Fn(f32, f32) -> Message,
        G: Fn(f32, f32) -> Message
{
    on_place: F,
    on_drag: G,
    data: &'a [Complex32],
    scale: f32
}

impl<'a, F, G, Message> Spectrum<'a, F, G, Message>
    where F: Fn(f32, f32) -> Message,
        G: Fn(f32, f32) -> Message
{
    pub fn new(on_place: F, on_drag: G, data: &'a [Complex32], scale: f32) -> Self
    {
        return Self {
            on_place,
            on_drag,
            data,
            scale
        };
    }
}

impl<'a, F, G, Message> Program<Message> for Spectrum<'a, F, G, Message>
    where F: Fn(f32, f32) -> Message,
        G: Fn(f32, f32) -> Message
{
    type State = bool;
    type Primitive = Lines;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: Rectangle) -> Self::Primitive
    {
        let s = self.scale / (self.data.len() as f32);
        let t = 300.0 / (self.data.len() as f32);
        return Lines::new(self.data.iter().skip(1).enumerate().map(|p|
        {
            let v = p.0 as f32 * t;
            let c = wave_length_colour(700.0 - v, 0.8);
            let amp = p.1.norm();
            return [amp * s, c.x, c.y, c.z];
        }).collect());
    }
    
    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
        _shell: &mut iced::advanced::Shell<'_, Message>) -> (Status, Option<Message>)
    {
        let w = bounds.width;
        let h = bounds.height;
        match event
        {
            Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left)) =>
            {
                if let Some(cursor_position) = cursor.position_over(bounds)
                {
                    *state = true;
                    let p = cursor_position - bounds.position();
                    let v = (h - p.y) / h;
                    let x = p.x / w;
                    
                    return (Status::Captured, Some((self.on_place)(x, v)));
                }
            },
            Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left)) =>
            {
                if *state
                {
                    *state = false;
                    return (Status::Captured, None);
                }
            },
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) =>
            {
                if *state
                {
                    let p = position - bounds.position();
                    let p = Point::new(p.x.clamp(0.0, bounds.width), p.y.clamp(0.0, bounds.height));
                    let v = (h - p.y) / h;
                    let x = p.x / w;
                    
                    return (Status::Captured, Some((self.on_drag)(x, v)));
                }
            },
            _ => {}
        }
        
        return (Status::Ignored, None);
    }
}