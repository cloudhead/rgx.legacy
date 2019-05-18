#![deny(clippy::all, clippy::use_self)]

extern crate cgmath;
extern crate shaderc;
extern crate wgpu;

use std::ops::Range;

use std::{mem, ptr};

///////////////////////////////////////////////////////////////////////////////
/// Draw
///////////////////////////////////////////////////////////////////////////////

pub trait Draw {
    fn draw(&self, binding: &BindingGroup, pass: &mut Pass);
}

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
/// Resource
///////////////////////////////////////////////////////////////////////////////

/// Anything that needs to be submitted to the GPU before the frame starts.
pub trait Resource {
    fn prepare(&self, encoder: &mut wgpu::CommandEncoder);
}

///////////////////////////////////////////////////////////////////////////////
/// BindingGroup
///////////////////////////////////////////////////////////////////////////////

/// A group of bindings.
pub struct BindingGroup {
    wgpu: wgpu::BindGroup,
    set_index: u32,
}

impl BindingGroup {
    fn new(set_index: u32, wgpu: wgpu::BindGroup) -> Self {
        Self { set_index, wgpu }
    }
}

/// The layout of a 'BindingGroup'.
pub struct BindingGroupLayout {
    wgpu: wgpu::BindGroupLayout,
    size: usize,
    set_index: u32,
}

impl BindingGroupLayout {
    fn new(set_index: u32, layout: wgpu::BindGroupLayout, size: usize) -> Self {
        Self {
            wgpu: layout,
            size,
            set_index,
        }
    }
}

/// A trait representing a resource that can be bound.
pub trait Bind {
    fn binding(&self, index: u32) -> wgpu::Binding;
}

///////////////////////////////////////////////////////////////////////////////
/// Uniforms
///////////////////////////////////////////////////////////////////////////////

/// A uniform buffer that can be bound in a 'BindingGroup'.
pub struct UniformBuffer {
    wgpu: wgpu::Buffer,
    size: usize,
}

impl Bind for UniformBuffer {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::Buffer {
                buffer: &self.wgpu,
                range: 0..(self.size as u32),
            },
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Texturing
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub struct Texture {
    wgpu: wgpu::Texture,
    view: wgpu::TextureView,
    extent: wgpu::Extent3d,
    buffer: wgpu::Buffer,

    pub w: u32,
    pub h: u32,
}

impl Bind for Texture {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}
impl Resource for &Texture {
    fn prepare(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &self.buffer,
                offset: 0,
                row_pitch: 4 * self.w,
                image_height: self.h,
            },
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                level: 0,
                slice: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            self.extent,
        );
    }
}

pub struct Sampler {
    wgpu: wgpu::Sampler,
}

impl Bind for Sampler {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::Sampler(&self.wgpu),
        }
    }
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
    pub size: u32,
    wgpu: wgpu::Buffer,
}

impl Draw for VertexBuffer {
    fn draw(&self, binding: &BindingGroup, pass: &mut Pass) {
        pass.apply_binding(binding, &[]);
        pass.set_vertex_buffer(&self);
        pass.draw_buffer(0..self.size, 0..1);
    }
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

/// Describes a 'VertexBuffer' layout.
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
/// Pipeline Bindings
///////////////////////////////////////////////////////////////////////////////

/// A binding type.
pub enum BindingType {
    UniformBuffer,
    Sampler,
    SampledTexture,
}

impl BindingType {
    fn to_wgpu(&self) -> wgpu::BindingType {
        match self {
            BindingType::UniformBuffer => wgpu::BindingType::UniformBufferDynamic,
            BindingType::SampledTexture => wgpu::BindingType::SampledTexture,
            BindingType::Sampler => wgpu::BindingType::Sampler,
        }
    }
}

pub struct Binding {
    pub binding: BindingType,
    pub stage: ShaderStage,
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline
///////////////////////////////////////////////////////////////////////////////

pub struct Pipeline {
    wgpu: wgpu::RenderPipeline,

    pub layout: PipelineLayout,
    pub vertex_layout: VertexLayout,
}

impl<'a> PipelineLike<'a> for Pipeline {
    type PrepareContext = ();
    type Uniforms = ();

    fn setup(pipeline: Self, _dev: &Device, _w: u32, _h: u32) -> Self {
        pipeline
    }

    fn apply(&self, pass: &mut Pass) {
        pass.wgpu.set_pipeline(&self.wgpu);
    }

    fn resize(&mut self, _w: u32, _h: u32) {}

    fn prepare(&'a self, _unused: ()) -> Option<(&'a UniformBuffer, Vec<()>)> {
        None
    }
}

pub struct Set<'a>(pub &'a [Binding]);

pub struct PipelineLayout {
    pub sets: Vec<BindingGroupLayout>,
}

pub trait PipelineLike<'a> {
    type PrepareContext;
    type Uniforms: Copy + 'static;

    fn setup(pip: Pipeline, dev: &Device, w: u32, h: u32) -> Self;
    fn apply(&self, pass: &mut Pass);
    fn resize(&mut self, w: u32, h: u32);
    fn prepare(
        &'a self,
        t: Self::PrepareContext,
    ) -> Option<(&'a UniformBuffer, Vec<Self::Uniforms>)>;
}

pub struct PipelineDescription<'a> {
    pub vertex_layout: &'a [VertexFormat],
    pub pipeline_layout: &'a [Set<'a>],
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
}

///////////////////////////////////////////////////////////////////////////////
/// Frame
///////////////////////////////////////////////////////////////////////////////

pub struct Frame<'a> {
    encoder: mem::ManuallyDrop<wgpu::CommandEncoder>,
    texture: wgpu::SwapChainOutput<'a>,
    device: &'a mut Device,
}

impl<'a> Drop for Frame<'a> {
    fn drop(&mut self) {
        let e = unsafe { mem::ManuallyDrop::into_inner(ptr::read(&self.encoder)) };
        self.device.submit(&[e.finish()]);
    }
}

impl<'a> Frame<'a> {
    pub fn new(
        encoder: wgpu::CommandEncoder,
        texture: wgpu::SwapChainOutput<'a>,
        device: &'a mut Device,
    ) -> Frame<'a> {
        Frame {
            texture,
            device,
            encoder: mem::ManuallyDrop::new(encoder),
        }
    }

    pub fn prepare<T>(&mut self, pip: &'a T, p: T::PrepareContext)
    where
        T: PipelineLike<'a>,
    {
        if let Some((buf, unifs)) = pip.prepare(p) {
            self.update_uniform_buffer(buf, unifs.as_slice());
        }
    }

    pub fn pass(&mut self, clear: Rgba) -> Pass {
        Pass::begin(&mut self.encoder, &self.texture.view, clear)
    }

    fn update_uniform_buffer<T>(&mut self, u: &UniformBuffer, buf: &[T])
    where
        T: 'static + Copy,
    {
        let src = self
            .device
            .device
            .create_buffer_mapped::<T>(
                buf.len(),
                wgpu::BufferUsageFlags::UNIFORM
                    | wgpu::BufferUsageFlags::TRANSFER_SRC
                    | wgpu::BufferUsageFlags::MAP_WRITE,
            )
            .fill_from_slice(buf);

        self.encoder.copy_buffer_to_buffer(
            &src,
            0,
            &u.wgpu,
            0,
            (std::mem::size_of::<T>() * buf.len()) as u32,
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pass
///////////////////////////////////////////////////////////////////////////////

pub struct Pass<'a> {
    wgpu: wgpu::RenderPass<'a>,
}

impl<'a> Pass<'a> {
    pub fn begin(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear_color: Rgba,
    ) -> Self {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: clear_color.to_wgpu(),
            }],
            depth_stencil_attachment: None,
        });
        Pass { wgpu: pass }
    }
    pub fn apply_pipeline<T>(&mut self, pipeline: &T)
    where
        T: PipelineLike<'a>,
    {
        pipeline.apply(self);
    }
    pub fn apply_binding(&mut self, group: &BindingGroup, offsets: &[u32]) {
        self.wgpu
            .set_bind_group(group.set_index, &group.wgpu, offsets);
    }
    pub fn set_index_buffer(&mut self, index_buf: &IndexBuffer) {
        self.wgpu.set_index_buffer(&index_buf.wgpu, 0)
    }
    pub fn set_vertex_buffer(&mut self, vertex_buf: &VertexBuffer) {
        self.wgpu.set_vertex_buffers(&[(&vertex_buf.wgpu, 0)])
    }
    pub fn draw<T: Draw>(&mut self, drawable: &T, binding: &BindingGroup) {
        drawable.draw(binding, self);
    }
    pub fn draw_buffer(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw(indices, instances)
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw_indexed(indices, 0, instances)
    }
}

pub fn frame<'a>(swap_chain: &'a mut wgpu::SwapChain, device: &'a mut Device) -> Frame<'a> {
    let texture = swap_chain.get_next_texture();
    let encoder = device.create_command_encoder();
    Frame::new(encoder, texture, device)
}

fn swap_chain_descriptor(width: u32, height: u32) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width,
        height,
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Renderer
///////////////////////////////////////////////////////////////////////////////

pub struct Renderer {
    pub device: Device,
    swap_chain: wgpu::SwapChain,
}

impl Renderer {
    pub fn new(window: &wgpu::winit::Window) -> Self {
        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        let device = Device::new(window);
        let swap_chain = device.create_swap_chain(size.width as u32, size.height as u32);

        Self { device, swap_chain }
    }

    pub fn texture(&self, texels: &[u8], w: u32, h: u32) -> Texture {
        self.device.create_texture(texels, w, h)
    }

    pub fn sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        self.device.create_sampler(min_filter, mag_filter)
    }

    pub fn pipeline<T>(&self, desc: PipelineDescription, w: u32, h: u32) -> T
    where
        T: PipelineLike<'static>,
    {
        let pip_layout = self.device.create_pipeline_layout(desc.pipeline_layout);
        let vertex_layout = VertexLayout::from(desc.vertex_layout);
        let vs = self
            .device
            .create_shader("shader.vert", desc.vertex_shader, ShaderStage::Vertex);
        let fs =
            self.device
                .create_shader("shader.frag", desc.fragment_shader, ShaderStage::Fragment);

        T::setup(
            self.device
                .create_pipeline(pip_layout, vertex_layout, &vs, &fs),
            &self.device,
            w,
            h,
        )
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn resize(&mut self, w: u32, h: u32) {
        self.swap_chain = self.device.create_swap_chain(w, h);
    }

    pub fn frame(&mut self) -> Frame {
        frame(&mut self.swap_chain, &mut self.device)
    }

    pub fn prepare<T: Resource>(&mut self, resources: &[T]) {
        let mut encoder = self.device.create_command_encoder();
        for r in resources.iter() {
            r.prepare(&mut encoder);
        }
        self.device.submit(&[encoder.finish()]);
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Device
///////////////////////////////////////////////////////////////////////////////

pub struct Device {
    device: wgpu::Device,
    surface: wgpu::Surface,
}

impl Device {
    pub fn new(window: &wgpu::winit::Window) -> Self {
        let instance = wgpu::Instance::new();
        let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
            power_preference: wgpu::PowerPreference::LowPower,
        });
        let surface = instance.create_surface(&window);

        Self {
            device: adapter.create_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
            }),
            surface,
        }
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 })
    }

    pub fn create_swap_chain(&self, w: u32, h: u32) -> wgpu::SwapChain {
        let desc = swap_chain_descriptor(w, h);
        self.device.create_swap_chain(&self.surface, &desc)
    }

    pub fn create_pipeline_layout(&self, ss: &[Set]) -> PipelineLayout {
        let mut sets = Vec::new();
        for (i, s) in ss.iter().enumerate() {
            sets.push(self.create_binding_group_layout(i as u32, s.0))
        }
        PipelineLayout { sets }
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

    pub fn create_texture(&self, texels: &[u8], w: u32, h: u32) -> Texture {
        assert_eq!(
            texels.len() as u32,
            w * h * 4,
            "wrong texture width or height given"
        );

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

        let buf = self
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsageFlags::TRANSFER_SRC)
            .fill_from_slice(&texels);

        Texture {
            wgpu: texture,
            view: texture_view,
            extent: texture_extent,
            buffer: buf,
            w,
            h,
        }
    }

    pub fn create_binding_group(
        &self,
        layout: &BindingGroupLayout,
        binds: &[&dyn Bind],
    ) -> BindingGroup {
        assert_eq!(
            binds.len(),
            layout.size,
            "layout slot count does not match bindings"
        );

        let mut bindings = Vec::new();

        for (i, b) in binds.iter().enumerate() {
            bindings.push(b.binding(i as u32));
        }

        BindingGroup::new(
            layout.set_index,
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout.wgpu,
                bindings: bindings.as_slice(),
            }),
        )
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
            size: vertices.len() as u32,
        }
    }

    pub fn create_uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        UniformBuffer {
            size: std::mem::size_of::<T>(),
            wgpu: self
                .device
                .create_buffer_mapped::<T>(
                    buf.len(),
                    wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
                )
                .fill_from_slice(buf),
        }
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

    pub fn create_binding_group_layout(&self, index: u32, slots: &[Binding]) -> BindingGroupLayout {
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
        BindingGroupLayout::new(index, layout, bindings.len())
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn submit(&mut self, cmds: &[wgpu::CommandBuffer]) {
        self.device.get_queue().submit(cmds);
    }

    // PRIVATE API ////////////////////////////////////////////////////////////

    fn create_pipeline(
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
}
