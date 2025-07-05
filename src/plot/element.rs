use std::ops::Range;
use std::fmt::Debug;

use bytemuck::NoUninit;
use iced::mouse::Button;
use iced::widget::{shader::{Event, Program}, shader};
use iced::advanced::graphics::core::event::Status;
use iced::widget::Shader;
use iced::{Point, Rectangle};
use zene_structs::Vector4;

use crate::plot::renderer::{PlotRender, TextureData};

pub fn plotter<'a, S, F, G, D: TextureData, Message, const ID: usize>(on_size: Option<S>, on_place: F,
    on_drag: G, data: &'a [D], data_range: Range<f32>, colour: Vector4<f32>) -> Shader<Message, Plot<'a, S, F, G, D, Message, ID>>
    where S: Fn(usize) -> Message,
        F: Fn(usize, f32) -> Message,
        G: Fn(usize, f32) -> Message,
        D: Debug + Send + Sync + NoUninit + 'static
{
    let s = 1.0 / (data_range.end - data_range.start);
    let off = -data_range.start * s;
    let top = (data_range.end * s) + off;
    
    return shader(
        Plot {
            colour,
            on_size,
            on_place,
            on_drag,
            data,
            scale: s,
            uv_scale: top,
            uv_offset: off
        }
    );
}

pub struct Plot<'a, S, F, G, D: TextureData, Message, const ID: usize>
    where S: Fn(usize) -> Message,
        F: Fn(usize, f32) -> Message,
        G: Fn(usize, f32) -> Message
{
    colour: Vector4<f32>,
    on_size: Option<S>,
    on_place: F,
    on_drag: G,
    data: &'a [D],
    scale: f32,
    uv_scale: f32,
    uv_offset: f32
}

impl<'a, S, F, G, D: TextureData, Message, const ID: usize> Program<Message> for Plot<'a, S, F, G, D, Message, ID>
    where S: Fn(usize) -> Message,
        F: Fn(usize, f32) -> Message,
        G: Fn(usize, f32) -> Message,
        D: Debug + Send + Sync + NoUninit + 'static
{
    type State = bool;
    type Primitive = PlotRender<D, ID>;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: Rectangle) -> Self::Primitive
    {
        return PlotRender::new(
            self.data.to_vec(), self.colour,
            Vector4::new(0.0, 0.0, 0.0, 1.0), self.scale, self.uv_scale, self.uv_offset);
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
        
        if self.data.len() != width && self.on_size.is_some()
        {
            let f = self.on_size.as_ref().unwrap();
            shell.publish(f(width));
        }
        
        let hh = bounds.height;
        match event
        {
            Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left)) =>
            {
                if let Some(cursor_position) = cursor.position_over(bounds)
                {
                    *state = true;
                    let p = cursor_position - bounds.position();
                    let v = ((hh - p.y) * self.uv_scale / hh - self.uv_offset) / self.scale;
                    let x = p.x as usize;
                    
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
                    let p = Point::new(p.x.clamp(0.0, bounds.width - 1.0), p.y.clamp(0.0, bounds.height));
                    let v = ((hh - p.y) * self.uv_scale / hh - self.uv_offset) / self.scale;
                    let x = p.x as usize;
                    
                    return (Status::Captured, Some((self.on_drag)(x, v)));
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
        
        if *state
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