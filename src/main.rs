mod plot_element;
mod line_renderer;

use iced::{widget::{button, column, shader, slider, text}, Alignment, Element};
use plot_element::{Plot, PlotData};

#[derive(Debug, Clone)]
enum Message
{
    Increment,
    Set(u32),
    PlotSize(usize),
    PlotPoint(usize, f32),
    PlotLine(usize, f32)
}

#[derive(Debug, Clone, Default)]
struct State
{
    counter: u32,
    plot: PlotData,
    last_point: (usize, f32)
}

fn lerp_index(vec: &Vec<f32>, i: f32) -> f32
{
    let ld = i.floor();
    let l = ld as usize;
    let u = i.ceil() as usize;
    
    if u >= vec.len()
    {
        return vec[l];
    }
    let i = i - ld;
    
    let a = vec[l];
    let b = vec[u];
    return ((a - b) * i) + b;
}
fn remap(a: &Vec<f32>, b: &mut Vec<f32>)
{
    let al = a.len();
    // no data to remap
    if al == 0 { return; }
    
    let step = (al as f32) / (b.len() as f32);
    
    let mut fi = 0.0;
    for v in b.iter_mut()
    {
        *v = lerp_index(a, fi);
        fi += step;
    }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Increment => state.counter += 1,
        Message::Set(v) => state.counter = v,
        Message::PlotSize(size) =>
        {
            let old = &state.plot.points;
            let mut new = vec![0.0; size];
            
            remap(old, &mut new);
            
            state.plot.points = new;
        },
        Message::PlotPoint(i, v) => state.plot.points[i] = v,
        Message::PlotLine(i, v) =>
        {
            state.plot.points[i] = v;
            state.last_point = (i, v);
        }
    }
}

fn view(state: &State) -> Element<Message> {
    column![
        text(state.counter).size(20),
        button("Increment").on_press(Message::Increment),
        slider(0..=50, state.counter, Message::Set),
        shader(Plot::new(Message::PlotSize, Message::PlotPoint, Message::PlotLine, &state.plot))
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .into()
}

fn main() {
    let _ = iced::run("A cool counter", update, view);
}