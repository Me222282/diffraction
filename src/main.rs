mod plot_element;
mod line_renderer;

use iced::{widget::{button, column, slider, text}, Alignment, Element};

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Set(u32)
}

fn update(counter: &mut u32, message: Message) {
    match message {
        Message::Increment => *counter += 1,
        Message::Set(v) => *counter = v
    }
}

fn view(counter: &u32) -> Element<Message> {
    column![
        text(counter).size(20),
        button("Increment").on_press(Message::Increment),
        slider(0..=50, *counter, Message::Set)
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .into()
}

fn main() {
    let _ = iced::run("A cool counter", update, view);
}