struct Globals
{
    colour: vec4<f32>,
    h_width: f32
};

@group(0) @binding(0)
var<uniform> uniform_data: Globals;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @location(0) in: f32) -> @builtin(position) vec4<f32>
{
    let pos = vec4<f32>((f32(in_vertex_index) / uniform_data.h_width) - 1.0, (in * 1.99) - 1.0, 0.0, 1.0);
    // let pos = vec4<f32>(f32(in_vertex_index), f32(in_vertex_index), 0.0, 0.0);
    return pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32>
{
    return uniform_data.colour;
}