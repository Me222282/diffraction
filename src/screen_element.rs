use iced::widget::{shader::Program, shader};
use iced::widget::Shader;
use iced::Rectangle;

use crate::screen_renderer::{Screen, SCREEN_SIZE};

pub fn screen<'a>(colours: &'a [[f32; 3]]) -> Shader<(), ScreenEl<'a>>
{
    return shader(
        ScreenEl { colours }
    ).width(SCREEN_SIZE as f32);
}

pub struct ScreenEl<'a>
{
    colours: &'a [[f32; 3]]
}

impl<'a> Program<()> for ScreenEl<'a>
{
    type State = ();
    type Primitive = Screen;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: Rectangle) -> Self::Primitive
    {
        return Screen::new(self.colours.to_vec());
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