struct Globals
{
    matrix: mat4x4<f32>,
    colour: vec4<f32>
};

@group(0) @binding(0)
var<uniform> uniform_data: Globals;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @location(0) in: f32) -> @builtin(position) vec4<f32>
{
    let pos = vec4<f32>((f32(in_vertex_index) / 50.0) - 1.0, (in * 2.0) - 1.0, 0.0, 1.0);
    // let pos = vec4<f32>(f32(in_vertex_index), f32(in_vertex_index), 0.0, 0.0);
    return pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32>
{
    return uniform_data.colour;
}