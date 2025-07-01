use iced::widget::{shader::Program, shader};
use iced::widget::Shader;
use iced::Rectangle;
use zene_structs::Vector2;

use crate::scene::LineData;
use crate::scene_renderer::SceneRender;

pub fn scene<'a, Mesaage>(lines: &'a [LineData], zoom: f32, pan: Vector2<f32>) -> Shader<Mesaage, SceneEl<'a>>
{
    return shader(
        SceneEl { lines, zoom, pan }
    );
}

pub struct SceneEl<'a>
{
    lines: &'a [LineData],
    zoom: f32,
    pan: Vector2<f32>
}

impl<'a, Message> Program<Message> for SceneEl<'a>
{
    type State = ();
    type Primitive = SceneRender;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: Rectangle) -> Self::Primitive
    {
        return SceneRender::new(self.lines.to_vec(), self.zoom, self.pan);
    }
    
    // fn update(
    //     &self,
    //     state: &mut Self::State,
    //     event: Event,
    //     bounds: Rectangle,
    //     cursor: iced::advanced::mouse::Cursor,
    //     shell: &mut iced::advanced::Shell<'_, Message>) -> (Status, Option<Message>)
    // {
    //     let width = bounds.width as usize;
        
    //     if self.data.len() != width && self.on_size.is_some()
    //     {
    //         let f = self.on_size.as_ref().unwrap();
    //         shell.publish(f(width));
    //     }
        
    //     let hh = bounds.height;
    //     match event
    //     {
    //         Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left)) =>
    //         {
    //             if let Some(cursor_position) = cursor.position_over(bounds)
    //             {
    //                 *state = true;
    //                 let p = cursor_position - bounds.position();
    //                 let v = ((hh - p.y) * self.uv_scale / hh - self.uv_offset) / self.scale;
    //                 let x = p.x as usize;
                    
    //                 return (Status::Captured, Some((self.on_place)(x, v)));
    //             }
    //         },
    //         Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left)) =>
    //         {
    //             if *state
    //             {
    //                 *state = false;
    //                 return (Status::Captured, None);
    //             }
    //         },
    //         Event::Mouse(iced::mouse::Event::CursorMoved { position }) =>
    //         {
    //             if *state
    //             {
    //                 let p = position - bounds.position();
    //                 let p = Point::new(p.x.clamp(0.0, bounds.width - 1.0), p.y.clamp(0.0, bounds.height));
    //                 let v = ((hh - p.y) * self.uv_scale / hh - self.uv_offset) / self.scale;
    //                 let x = p.x as usize;
                    
    //                 return (Status::Captured, Some((self.on_drag)(x, v)));
    //             }
    //         },
    //         _ => {}
    //     }
        
    //     return (Status::Ignored, None);
    // }
    
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