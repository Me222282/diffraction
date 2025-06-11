struct VertexIn
{
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>
}

struct VertexOut
{
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: f32,
}

@group(0) @binding(0)
var colours: texture_1d<f32>;
@group(0) @binding(1)
var data_sampler: sampler;

@vertex
fn vs_main(in: VertexIn) -> VertexOut
{
    var out: VertexOut;
    out.pos = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv.x;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32>
{
    return vec4<f32>(textureSample(colours, data_sampler, in.uv).rgb, 1.0);
}