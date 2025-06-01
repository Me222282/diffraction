struct Globals
{
    background: vec4<f32>,
    // half on height, full on width
    h_size: vec2<f32>
};

struct VertexIn
{
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>
}

struct VertexOut
{
    @builtin(position) pos: vec4<f32>,
    @location(0) sample: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniform_data: Globals;
@group(0) @binding(1)
var data_source: texture_1d<f32>;
// @group(0) @binding(2)
// var data_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) i: u32, in: VertexIn) -> VertexOut
{
    var out: VertexOut;
    out.pos = vec4<f32>(in.position, 0.0, 1.0);
    out.sample = vec2<f32>(in.uv.x * (uniform_data.h_size.x - 1.0), in.uv.y * uniform_data.h_size.y);
    return out;
}

fn load_sample(index: u32) -> i32
{
    let s = textureLoad(data_source, index + 1u, 0).a * uniform_data.h_size.y * 0.999;
    return get_current(s);
}
fn get_current(v: f32) -> i32
{
    return i32(v + uniform_data.h_size.y) - i32(uniform_data.h_size.y);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32>
{
    let index = u32(in.sample.x);
    // let v = load_sample(index);
    let sample = textureLoad(data_source, index + 1u, 0);
    let v = get_current(sample.a * uniform_data.h_size.y * 0.999);
    let c = get_current(in.sample.y);
    let colour = vec4<f32>(sample.rgb, 1.0);
    
    // outside plot
    if c > v && c >= 0
    {
        if index > 0u
        {
            // joining points
            let lv = load_sample(index - 1u);
            if c < lv
            {
                return colour;
            }
        }
        if c != 0
        {
            return uniform_data.background;
        }
    }
    if c <= 0 && c < v
    {
        if index > 0u
        {
            // joining points
            let lv = load_sample(index - 1u);
            if c > lv
            {
                return colour;
            }
        }
        if c != 0
        {
            return uniform_data.background;
        }
    }
    
    // on plot line
    if c == v
    {
        return colour;
    }
    if index > 0u
    {
        // joining points
        let lv = load_sample(index - 1u);
        if (c > lv && c > 0) || (c < lv && c < 0)
        {
            return colour;
        }
    }
    
    // vertical lines
    if (index % 10u == 0u)
    {
        return mix(uniform_data.background, colour, 0.6);
    }
    
    return mix(uniform_data.background, colour, 0.3);
}