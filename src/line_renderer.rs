use iced::widget::shader::wgpu::util::{BufferInitDescriptor, DeviceExt};
use iced::widget::shader::wgpu::*;
use iced::widget::shader::Primitive;
use iced::Rectangle;
use num::Zero;
use zene_structs::{Matrix4, Vector4};

#[derive(Debug)]
pub struct Lines
{
    colour: Vector4<f32>,
    bounds: Rectangle,
    data: Vec<f32>,
    size: u32
}

impl Lines
{
    pub fn new(colour: Vector4<f32>, data: Vec<f32>, bounds: Rectangle) -> Self
    {
        let size = data.len() as u32;
        return Self {
            colour,
            bounds,
            data,
            size
        };
    }
}

impl Primitive for Lines
{
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        // custom pipelines go here
        storage: &mut iced::widget::shader::Storage,
        _: &Rectangle,
        viewport: &iced::widget::shader::Viewport)
    {
        let pipe = storage.get_mut::<LinePipe>();
        let pipe = match pipe
        {
            Some(lp) => lp,
            None =>
            {
                let lp = LinePipe::new(device, format);
                storage.store(lp);
                storage.get_mut::<LinePipe>().unwrap()
            },
        };
        
        let bounds = self.bounds;
        // let mat = Matrix4::<f32>::from(viewport.projection().as_ref()) *
        //     Matrix4::<f32>::create_scale_2(bounds.width / (self.size as f32), bounds.height) *
        //     Matrix4::<f32>::create_translation_2([bounds.x, bounds.y].into());
        let mat = Matrix4::<f32>::identity();
        
        let uni_dat = Uniform { matrix: mat, colour: self.colour };
        queue.write_buffer(&pipe.uniform_buffer, 0,
            bytemuck::cast_slice(&[uni_dat]));
        
        pipe.write_vertex(device, queue, &self.data);
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &Rectangle<u32>)
    {
        let pipe = storage.get::<LinePipe>();
        match pipe
        {
            Some(pipe) =>
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("lines.render"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &target,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None
                });
                
                render_pass.set_scissor_rect(clip_bounds.x, clip_bounds.y, clip_bounds.width, clip_bounds.height);
                render_pass.set_viewport(clip_bounds.x as f32,
                    clip_bounds.y as f32,
                    clip_bounds.width as f32,
                    clip_bounds.height as f32, 0.0, 1.0);
                render_pass.set_pipeline(&pipe.render_pipeline);
                render_pass.set_bind_group(0, &pipe.bind_group, &[]);
                render_pass.set_vertex_buffer(0, pipe.vertex_buffer.slice(..));
                render_pass.draw(0..self.size, 0..1);
            },
            None => {},
        };
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
struct Uniform
{
    matrix: Matrix4<f32>,
    colour: Vector4<f32>
}
unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}

struct LinePipe
{
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    vertices: usize
}

impl LinePipe
{
    pub fn write_vertex(&mut self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        data: &[f32])
    {
        if self.vertices < data.len()
        {
            self.vertices = data.len();
            // recreate buffer
            self.vertex_buffer = device.create_buffer_init(
                &util::BufferInitDescriptor {
                    label: Some("lines.verts"),
                    contents: bytemuck::cast_slice(data),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST
                }
            );
            return;
        }
        
        queue.write_buffer(&self.vertex_buffer, 0,
            bytemuck::cast_slice(data));
    }
    
    pub fn new(
        device: &iced::widget::shader::wgpu::Device,
        format: iced::widget::shader::wgpu::TextureFormat) -> LinePipe
    {
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lines.uniform"),
            contents: bytemuck::cast_slice(&[Uniform { matrix: Matrix4::<f32>::identity(), colour: Vector4::<f32>::zero() }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("lines.uniform.bind"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("lines.uniform.group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ]
        });
        
        let shader = device.create_shader_module(include_wgsl!("line_shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("lines.rp.lay"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[]
        });
        
        let vertex_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("lines.verts"),
                contents: bytemuck::cast_slice(&[0f32, 0f32]),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST
            }
        );
        
        let buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<f32>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32]
        };
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("lines.pipe"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                // specify vertex buffer layout
                buffers: &[buffer_layout]
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })]
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineStrip,
                front_face: FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None
        });
        
        return Self {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group: uniform_bind_group,
            vertices: 2
        };
    }
}