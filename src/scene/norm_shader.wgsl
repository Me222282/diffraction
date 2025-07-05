struct VertexIn
{
    @location(0) position: vec2<f32>,
    @location(1) colour: vec4<f32>
}

struct VertexOut
{
    @builtin(position) pos: vec4<f32>,
    @location(1) colour: vec4<f32>,
}

struct Uniform
{
    pan: vec2<f32>,
    scale: vec2<f32>
}

@group(0) @binding(0)
var<uniform> uni: Uniform;

@vertex
fn vs_main(in: VertexIn, @builtin(vertex_index) i: u32) -> VertexOut
{
    var out: VertexOut;
    out.pos = vec4<f32>((in.position * uni.scale) + uni.pan, 0.0, 1.0);
    out.colour = in.colour;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32>
{
    return in.colour;
}