use backend::wave_length_colour;
use iced::widget::shader::{Event, Program};
use iced::advanced::graphics::core::event::Status;
use iced::Rectangle;
use num::complex::Complex32;

use crate::spectrum_renderer::Lines;

pub struct Spectrum<'a>
{
    data: &'a [Complex32],
    scale: f32
}

impl<'a> Spectrum<'a>
{
    pub fn new(data: &'a [Complex32], scale: f32) -> Self
    {
        return Self { data, scale };
    }
}

impl<'a, Message> Program<Message> for Spectrum<'a>
{
    type State = ();
    type Primitive = Lines;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: Rectangle) -> Self::Primitive
    {
        let s = self.scale / (self.data.len() as f32);
        let t = 300.0 / (self.data.len() as f32);
        return Lines::new(self.data.iter().enumerate().map(|p|
        {
            let v = p.0 as f32 * t;
            let c = wave_length_colour(700.0 - v, 0.8);
            let amp = p.1.norm();
            return [amp * s, c.x, c.y, c.z];
        }).collect());
    }
    
    fn update(
        &self,
        _state: &mut Self::State,
        _event: Event,
        _bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
        _shell: &mut iced::advanced::Shell<'_, Message>) -> (Status, Option<Message>)
    {
        return (Status::Ignored, None);
    }
}