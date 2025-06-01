use iced::widget::shader::{Event, Program};
use iced::advanced::graphics::core::event::Status;
use iced::Rectangle;
use num::complex::Complex32;
use zene_structs::{Vector3, Vector};

use crate::spectrum_renderer::Lines;

pub struct Spectrum<'a>
{
    data: &'a [Complex32]
}

impl<'a> Spectrum<'a>
{
    pub fn new(data: &'a [Complex32]) -> Self
    {
        return Self { data };
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
        let c0 = Vector3::<f32>::new(1.0, 1.0, 0.0);
        let c1 = Vector3::<f32>::new(0.0, 1.0, 1.0);
        let s = 1.0 / (self.data.len() as f32);
        let mut ma = 0.0;
        return Lines::new(self.data.iter().enumerate().map(|p|
        {
            let v = p.0 as f32 * s;
            let c = c0.lerp(c1, v);
            let amp = p.1.norm();
            if amp > ma { ma = amp; }
            return [amp, c.x, c.y, c.z];
        }).collect(), 1.0 / ma);
    }
    
    fn update(
        &self,
        _state: &mut Self::State,
        _event: Event,
        _bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
        _shell: &mut iced::advanced::Shell<'_, Message>) -> (Status, Option<Message>)
    {
        // let width = bounds.width as usize;
        
        // if self.data.len() != width
        // {
        //     shell.publish((self.on_size)(width));
        // }
        
        return (Status::Ignored, None);
    }
}