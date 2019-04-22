#![deny(clippy::all, clippy::use_self)]

extern crate cgmath;
extern crate shaderc;
extern crate wgpu;

use std::ops::Range;
use std::rc::*;

///////////////////////////////////////////////////////////////////////////////
/// Rgba
///////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Shaders
///////////////////////////////////////////////////////////////////////////////

pub struct Shader {
    module: wgpu::ShaderModule,
}

#[derive(Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderStage {
    fn to_wgpu(&self) -> wgpu::ShaderStageFlags {
        match self {
            ShaderStage::Vertex => wgpu::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => wgpu::ShaderStageFlags::COMPUTE,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Uniforms
///////////////////////////////////////////////////////////////////////////////

pub struct Uniforms {
    wgpu: wgpu::BindGroup,
    set_index: u32,
}

impl Uniforms {
    fn new(set_index: u32, layout: wgpu::BindGroup) -> Self {
        Self {
            set_index,
            wgpu: layout,
        }
    }
}

pub struct UniformsLayout {
    wgpu: wgpu::BindGroupLayout,
    size: usize,
    set_index: u32,
}

impl UniformsLayout {
    fn new(set_index: u32, layout: wgpu::BindGroupLayout, size: usize) -> Self {
        Self {
            wgpu: layout,
            size,
            set_index,
        }
    }
}

pub struct UniformBuffer {
    wgpu: wgpu::Buffer,
    size: usize,
}

///////////////////////////////////////////////////////////////////////////////
/// Texturing
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub struct Texture {
    wgpu: wgpu::Texture,
    view: wgpu::TextureView,

    pub w: u32,
    pub h: u32,
}

pub struct Sampler {
    wgpu: wgpu::Sampler,
}

#[derive(Debug)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    fn to_wgpu(&self) -> wgpu::FilterMode {
        match self {
            Filter::Nearest => wgpu::FilterMode::Nearest,
            Filter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Vertex/Index Buffers
///////////////////////////////////////////////////////////////////////////////

pub struct VertexBuffer {
    wgpu: wgpu::Buffer,
}

pub struct IndexBuffer {
    wgpu: wgpu::Buffer,
}

#[derive(Clone, Copy)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    UByte4,
}

impl VertexFormat {
    // TODO: Use `const fn`
    fn bytesize(self) -> usize {
        match self {
            VertexFormat::Float => 4,
            VertexFormat::Float2 => 8,
            VertexFormat::Float3 => 12,
            VertexFormat::Float4 => 16,
            VertexFormat::UByte4 => 4,
        }
    }
    // TODO: Use `const fn`
    fn to_wgpu(self) -> wgpu::VertexFormat {
        match self {
            VertexFormat::Float => wgpu::VertexFormat::Float,
            VertexFormat::Float2 => wgpu::VertexFormat::Float2,
            VertexFormat::Float3 => wgpu::VertexFormat::Float3,
            VertexFormat::Float4 => wgpu::VertexFormat::Float4,
            VertexFormat::UByte4 => wgpu::VertexFormat::Uchar4Norm,
        }
    }
}

#[derive(Default)]
pub struct VertexLayout {
    wgpu_attrs: Vec<wgpu::VertexAttributeDescriptor>,
    size: usize,
}

impl VertexLayout {
    pub fn from(formats: &[VertexFormat]) -> Self {
        let mut vl = Self::default();
        for vf in formats {
            vl.wgpu_attrs.push(wgpu::VertexAttributeDescriptor {
                attribute_index: vl.wgpu_attrs.len() as u32,
                offset: vl.size as u32,
                format: vf.to_wgpu(),
            });
            vl.size += vf.bytesize();
        }
        vl
    }

    fn to_wgpu(&self) -> wgpu::VertexBufferDescriptor {
        wgpu::VertexBufferDescriptor {
            stride: self.size as u32,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: self.wgpu_attrs.as_slice(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Uniform Bindings
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum Uniform<'a> {
    Buffer(&'a UniformBuffer),
    Texture(&'a Texture),
    Sampler(&'a Sampler),
    Unbound(),
}

pub enum BindingType {
    UniformBuffer,
    Sampler,
    SampledTexture,
}

impl BindingType {
    fn to_wgpu(&self) -> wgpu::BindingType {
        match self {
            BindingType::UniformBuffer => wgpu::BindingType::UniformBuffer,
            BindingType::SampledTexture => wgpu::BindingType::SampledTexture,
            BindingType::Sampler => wgpu::BindingType::Sampler,
        }
    }
}

pub struct Binding {
    pub binding: BindingType,
    pub stage: ShaderStage,
}

pub struct UniformsBinding<'a> {
    slots: Vec<Uniform<'a>>,
    layout: &'a UniformsLayout,
}

impl<'a> UniformsBinding<'a> {
    pub fn from(layout: &'a UniformsLayout) -> UniformsBinding<'a> {
        UniformsBinding {
            slots: vec![Uniform::Unbound(); layout.size],
            layout,
        }
    }
}

impl<'a> std::ops::Index<usize> for UniformsBinding<'a> {
    type Output = Uniform<'a>;

    fn index(&self, index: usize) -> &Uniform<'a> {
        self.slots.index(index)
    }
}

impl<'a> std::ops::IndexMut<usize> for UniformsBinding<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Uniform<'a> {
        // TODO: Better error when out of bounds.
        self.slots.index_mut(index)
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline, Pass
///////////////////////////////////////////////////////////////////////////////

pub struct Pipeline {
    wgpu: wgpu::RenderPipeline,
    pub layout: PipelineLayout,
    pub vertex_layout: VertexLayout,
}

pub struct Set<'a>(pub &'a [Binding]);

pub struct PipelineLayout {
    pub sets: Vec<UniformsLayout>,
}

pub struct Pass<'a> {
    wgpu: wgpu::RenderPass<'a>,
}

impl<'a> Pass<'a> {
    pub fn new(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear_color: Rgba,
    ) -> Self {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: clear_color.to_wgpu(),
            }],
            depth_stencil_attachment: None,
        });
        Pass { wgpu: pass }
    }
    pub fn apply_pipeline(&mut self, pipeline: &Pipeline) {
        self.wgpu.set_pipeline(&pipeline.wgpu)
    }
    pub fn apply_uniforms(&mut self, uniforms: &Uniforms) {
        self.wgpu.set_bind_group(uniforms.set_index, &uniforms.wgpu)
    }
    pub fn set_index_buffer(&mut self, index_buf: &IndexBuffer) {
        self.wgpu.set_index_buffer(&index_buf.wgpu, 0)
    }
    pub fn set_vertex_buffer(&mut self, vertex_buf: &VertexBuffer) {
        self.wgpu.set_vertex_buffers(&[(&vertex_buf.wgpu, 0)])
    }
    pub fn draw(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw(indices, instances)
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw_indexed(indices, 0, instances)
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Context
///////////////////////////////////////////////////////////////////////////////

pub struct Context {
    pub device: wgpu::Device,

    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
}

impl Context {
    pub fn new(window: &wgpu::winit::Window) -> Self {
        let instance = wgpu::Instance::new();
        let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
            power_preference: wgpu::PowerPreference::LowPower,
        });
        let device = adapter.create_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
        });
        let surface = instance.create_surface(&window);

        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        let swap_chain_desc =
            swap_chain_descriptor(size.width.round() as u32, size.height.round() as u32);
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        Self {
            device,
            surface,
            swap_chain,
        }
    }

    pub fn create_shader(&self, name: &str, source: &str, stage: ShaderStage) -> Shader {
        let ty = match stage {
            ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
            ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
            ShaderStage::Compute => shaderc::ShaderKind::Compute,
        };

        let mut compiler = shaderc::Compiler::new().unwrap();
        let options = shaderc::CompileOptions::new().unwrap();

        let result = compiler.compile_into_spirv(source, ty, name, "main", Some(&options));

        let spv = match result {
            Ok(spv) => spv,
            Err(err) => match err {
                shaderc::Error::CompilationError(_, err) => {
                    panic!(err);
                }
                _ => unimplemented!(),
            },
        };
        Shader {
            module: self.device.create_shader_module(spv.as_binary_u8()),
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 })
    }
    pub fn submit_encoder(&mut self, bufs: &[wgpu::CommandBuffer]) {
        self.device.get_queue().submit(bufs);
    }

    pub fn create_pass<'a>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
        clear_color: Rgba,
    ) -> Pass<'a> {
        let chain_out = self.swap_chain.get_next_texture();
        Pass::new(encoder, &chain_out.view, clear_color)
    }

    pub fn create_framebuffer_texture(&mut self, w: u32, h: u32) -> Texture {
        let texture_extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_size: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsageFlags::SAMPLED | wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
        });
        let texture_view = texture.create_default_view();

        Texture {
            wgpu: texture,
            view: texture_view,
            w,
            h,
        }
    }

    pub fn create_texture(&mut self, texels: &[u8], w: u32, h: u32) -> Texture {
        let texture_extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_size: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsageFlags::SAMPLED | wgpu::TextureUsageFlags::TRANSFER_DST,
        });
        let texture_view = texture.create_default_view();
        let temp_buf = self
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsageFlags::TRANSFER_SRC)
            .fill_from_slice(&texels);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * w,
                image_height: h,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                level: 0,
                slice: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            texture_extent,
        );

        self.device.get_queue().submit(&[encoder.finish()]);
        Texture {
            wgpu: texture,
            view: texture_view,
            w,
            h,
        }
    }

    pub fn create_buffer<T>(&self, vertices: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        VertexBuffer {
            wgpu: self
                .device
                .create_buffer_mapped(vertices.len(), wgpu::BufferUsageFlags::VERTEX)
                .fill_from_slice(vertices),
        }
    }

    pub fn create_uniform_buffer<T>(&self, buf: T) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        UniformBuffer {
            size: std::mem::size_of::<T>(),
            wgpu: self
                .device
                .create_buffer_mapped::<T>(
                    1,
                    wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
                )
                .fill_from_slice(&[buf]),
        }
    }

    pub fn update_uniform_buffer<T>(
        &mut self,
        u: Rc<UniformBuffer>,
        buf: T,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: 'static + Copy,
    {
        let src = self
            .device
            .create_buffer_mapped::<T>(
                1,
                wgpu::BufferUsageFlags::UNIFORM
                    | wgpu::BufferUsageFlags::TRANSFER_SRC
                    | wgpu::BufferUsageFlags::MAP_WRITE,
            )
            .fill_from_slice(&[buf]);

        encoder.copy_buffer_to_buffer(&src, 0, &u.wgpu, 0, std::mem::size_of::<T>() as u32);
    }

    pub fn create_index(&self, indices: &[u16]) -> IndexBuffer {
        let index_buf = self
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsageFlags::INDEX)
            .fill_from_slice(indices);
        IndexBuffer { wgpu: index_buf }
    }

    pub fn create_sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        Sampler {
            wgpu: self.device.create_sampler(&wgpu::SamplerDescriptor {
                r_address_mode: wgpu::AddressMode::Repeat,
                s_address_mode: wgpu::AddressMode::Repeat,
                t_address_mode: wgpu::AddressMode::Repeat,
                mag_filter: mag_filter.to_wgpu(),
                min_filter: min_filter.to_wgpu(),
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                max_anisotropy: 0,
                compare_function: wgpu::CompareFunction::Always,
                border_color: wgpu::BorderColor::TransparentBlack,
            }),
        }
    }

    pub fn create_uniforms_layout(&self, index: u32, slots: &[Binding]) -> UniformsLayout {
        let mut bindings = Vec::new();

        for s in slots {
            bindings.push(wgpu::BindGroupLayoutBinding {
                binding: bindings.len() as u32,
                visibility: s.stage.to_wgpu(),
                ty: s.binding.to_wgpu(),
            });
        }
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: bindings.as_slice(),
            });
        UniformsLayout::new(index, layout, bindings.len())
    }

    pub fn create_uniforms(&self, bs: &UniformsBinding) -> Uniforms {
        let layout = bs.layout;
        let mut bindings = Vec::new();

        for (i, s) in bs.slots.iter().enumerate() {
            match s {
                Uniform::Buffer(unif) => {
                    bindings.push(wgpu::Binding {
                        binding: i as u32,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &unif.wgpu,
                            range: 0..(unif.size as u32),
                        },
                    });
                }
                Uniform::Texture(tex) => bindings.push(wgpu::Binding {
                    binding: i as u32,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                }),
                Uniform::Sampler(sam) => bindings.push(wgpu::Binding {
                    binding: i as u32,
                    resource: wgpu::BindingResource::Sampler(&sam.wgpu),
                }),
                Uniform::Unbound() => panic!("binding slot {} is unbound", i),
            };
        }
        Uniforms::new(
            layout.set_index,
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout.wgpu,
                bindings: bindings.as_slice(),
            }),
        )
    }

    pub fn create_binding(&self, layout: &UniformsLayout, us: &[Uniform]) -> Uniforms {
        let mut binding = UniformsBinding::from(layout);
        for (i, u) in us.iter().enumerate() {
            binding[i] = u.clone();
        }
        self.create_uniforms(&binding)
    }

    pub fn create_pipeline_layout(&self, ss: &[Set]) -> PipelineLayout {
        let mut sets = Vec::new();
        for (i, s) in ss.iter().enumerate() {
            sets.push(self.create_uniforms_layout(i as u32, s.0))
        }
        PipelineLayout { sets }
    }

    pub fn create_pipeline(
        &self,
        pipeline_layout: PipelineLayout,
        vertex_layout: VertexLayout,
        vs: &Shader,
        fs: &Shader,
    ) -> Pipeline {
        let vertex_attrs = vertex_layout.to_wgpu();

        let mut sets = Vec::new();
        for s in pipeline_layout.sets.iter() {
            sets.push(&s.wgpu);
        }
        let layout = &self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: sets.as_slice(),
            });

        let wgpu = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout,
                vertex_stage: wgpu::PipelineStageDescriptor {
                    module: &vs.module,
                    entry_point: "main",
                },
                fragment_stage: wgpu::PipelineStageDescriptor {
                    module: &fs.module,
                    entry_point: "main",
                },
                rasterization_state: wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                },
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWriteFlags::ALL,
                }],
                depth_stencil_state: None,
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_attrs],
                sample_count: 1,
            });

        Pipeline {
            layout: pipeline_layout,
            vertex_layout,
            wgpu,
        }
    }

    pub fn resize(&mut self, physical: wgpu::winit::dpi::PhysicalSize) {
        let swap_chain_descriptor = swap_chain_descriptor(
            physical.width.round() as u32,
            physical.height.round() as u32,
        );

        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &swap_chain_descriptor);
    }
}

fn swap_chain_descriptor(width: u32, height: u32) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width,
        height,
    }
}
