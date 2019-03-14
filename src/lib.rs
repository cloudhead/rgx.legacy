#![deny(clippy::all, clippy::use_self)]

extern crate cgmath;
extern crate env_logger;
extern crate shaderc;
extern crate wgpu;

use std::ops::Range;

pub struct Shader {
    pub module: wgpu::ShaderModule,
}

pub struct Sampler {
    pub wgpu: wgpu::Sampler,
}

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

pub struct Pipeline {
    pub wgpu: wgpu::RenderPipeline,
}

pub enum Command<'a> {
    UpdateUniformBuffer(&'a UniformBuffer, wgpu::Buffer, usize),
}

pub struct Context<'a> {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub swap_chain: wgpu::SwapChain,

    commands: Vec<Command<'a>>,
}

pub struct Bindings {
    pub wgpu: wgpu::BindGroup,
}

impl Bindings {
    pub fn new(layout: wgpu::BindGroup) -> Self {
        Self { wgpu: layout }
    }
}

pub struct BindingsBuilder<'a> {
    layout: &'a BindingsLayout,
    bindings: Vec<wgpu::Binding<'a>>,
    ctx: &'a Context<'a>,
}

impl<'a> BindingsBuilder<'a> {
    pub fn bind_uniforms(&mut self, unifs: &'a UniformBuffer) -> &mut Self {
        self.bindings.push(wgpu::Binding {
            binding: self.bindings.len() as u32,
            resource: wgpu::BindingResource::Buffer {
                buffer: &unifs.wgpu,
                range: 0..(unifs.size as u32),
            },
        });
        self
    }

    pub fn bind_texture(&mut self, texture: &'a Texture) -> &mut Self {
        self.bindings.push(wgpu::Binding {
            binding: self.bindings.len() as u32,
            resource: wgpu::BindingResource::TextureView(&texture.view),
        });
        self
    }

    pub fn bind_sampler(&mut self, sampler: &'a Sampler) -> &mut Self {
        self.bindings.push(wgpu::Binding {
            binding: self.bindings.len() as u32,
            resource: wgpu::BindingResource::Sampler(&sampler.wgpu),
        });
        self
    }

    pub fn build(&self) -> Bindings {
        let bind_group = self
            .ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.layout.wgpu,
                bindings: self.bindings.as_slice(),
            });
        Bindings::new(bind_group)
    }
}

pub struct BindingsLayout {
    pub wgpu: wgpu::BindGroupLayout,
    pub size: usize,
}

impl BindingsLayout {
    pub fn new(layout: wgpu::BindGroupLayout, size: usize) -> Self {
        Self { wgpu: layout, size }
    }
}

pub struct BindingsLayoutBuilder<'a> {
    bindings: Vec<wgpu::BindGroupLayoutBinding>,
    ctx: &'a Context<'a>,
}

impl<'a> BindingsLayoutBuilder<'a> {
    pub fn binding(&mut self, stage: &ShaderStage, bind_type: &BindingType) -> &mut Self {
        self.bindings.push(wgpu::BindGroupLayoutBinding {
            binding: self.bindings.len() as u32,
            visibility: stage.to_wgpu(),
            ty: bind_type.to_wgpu(),
        });
        self
    }

    pub fn build(&self) -> BindingsLayout {
        let layout = self
            .ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: self.bindings.as_slice(),
            });
        BindingsLayout::new(layout, self.bindings.len())
    }
}

pub struct Texture {
    pub wgpu: wgpu::Texture,
    pub view: wgpu::TextureView,
}

pub struct VertexBuffer {
    pub wgpu: wgpu::Buffer,
}

pub struct IndexBuffer {
    pub wgpu: wgpu::Buffer,
}

pub struct UniformBuffer {
    pub wgpu: wgpu::Buffer,
    pub size: usize,
}

#[derive(Clone)]
pub enum Binding<'a> {
    UniformBuffer(&'a UniformBuffer),
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

pub struct Slot {
    pub binding: BindingType,
    pub stage: ShaderStage,
}

pub struct BindingSlots<'a> {
    slots: Vec<Binding<'a>>,
    layout: &'a BindingsLayout,
}

impl<'a> BindingSlots<'a> {
    pub fn from(layout: &'a BindingsLayout) -> BindingSlots<'a> {
        BindingSlots {
            slots: vec![Binding::Unbound(); layout.size],
            layout,
        }
    }
}

impl<'a> std::ops::Index<usize> for BindingSlots<'a> {
    type Output = Binding<'a>;
    fn index(&self, index: usize) -> &Binding<'a> {
        self.slots.index(index)
    }
}

impl<'a> std::ops::IndexMut<usize> for BindingSlots<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Binding<'a> {
        self.slots.index_mut(index)
    }
}

#[derive(Clone, Copy)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
}

impl VertexFormat {
    // TODO: Use `const fn`
    pub fn bytesize(self) -> usize {
        match self {
            VertexFormat::Float => 4,
            VertexFormat::Float2 => 8,
            VertexFormat::Float3 => 12,
            VertexFormat::Float4 => 16,
        }
    }
    // TODO: Use `const fn`
    pub fn to_wgpu(self) -> wgpu::VertexFormat {
        match self {
            VertexFormat::Float => wgpu::VertexFormat::Float,
            VertexFormat::Float2 => wgpu::VertexFormat::Float2,
            VertexFormat::Float3 => wgpu::VertexFormat::Float3,
            VertexFormat::Float4 => wgpu::VertexFormat::Float4,
        }
    }
}

pub struct VertexAttr {
    index: u32,
    format: VertexFormat,
    offset: u32,
}

impl VertexAttr {
    pub fn to_wgpu(&self) -> wgpu::VertexAttributeDescriptor {
        wgpu::VertexAttributeDescriptor {
            attribute_index: self.index,
            format: self.format.to_wgpu(),
            offset: self.offset,
        }
    }
}

#[derive(Default)]
pub struct VertexLayout {
    pub wgpu_attrs: Vec<wgpu::VertexAttributeDescriptor>,
    pub size: usize,
}

pub struct VertexBufferDescriptor<'a> {
    pub wgpu: wgpu::VertexBufferDescriptor<'a>,
}

impl VertexLayout {
    pub fn from(formats: &[VertexFormat]) -> Self {
        let mut vl = Self::default();
        for vf in formats {
            vl.attr(*vf);
        }
        vl
    }

    fn attr(&mut self, format: VertexFormat) -> &mut Self {
        self.wgpu_attrs.push(wgpu::VertexAttributeDescriptor {
            attribute_index: self.wgpu_attrs.len() as u32,
            offset: self.size as u32,
            format: format.to_wgpu(),
        });
        self.size += format.bytesize();

        self
    }

    pub fn build(&self) -> VertexBufferDescriptor {
        VertexBufferDescriptor {
            wgpu: wgpu::VertexBufferDescriptor {
                stride: self.size as u32,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: self.wgpu_attrs.as_slice(),
            },
        }
    }
}

pub struct Pass<'a> {
    pub wgpu: wgpu::RenderPass<'a>,
}

impl<'a> Pass<'a> {
    pub fn apply_pipeline(&mut self, pipeline: &Pipeline) {
        self.wgpu.set_pipeline(&pipeline.wgpu)
    }
    pub fn apply_bindings(&mut self, bindings: &Bindings) {
        self.wgpu.set_bind_group(0, &bindings.wgpu)
    }
    pub fn set_index_buffer(&mut self, index_buf: &IndexBuffer) {
        self.wgpu.set_index_buffer(&index_buf.wgpu, 0)
    }
    pub fn set_vertex_buffer(&mut self, vertex_buf: &VertexBuffer) {
        self.wgpu.set_vertex_buffers(&[(&vertex_buf.wgpu, 0)])
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw_indexed(indices, 0, instances)
    }
}

pub struct Frame<'a> {
    view: &'a wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'a> Frame<'a> {
    pub fn begin_pass(&mut self) -> Pass {
        let pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.view,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::WHITE,
            }],
            depth_stencil_attachment: None,
        });
        Pass { wgpu: pass }
    }
}

impl<'a> Context<'a> {
    pub fn new(window: &wgpu::winit::Window) -> Self {
        env_logger::init();

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
        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: size.width as u32,
                height: size.height as u32,
            },
        );

        let commands = Vec::new();

        Self {
            instance,
            adapter,
            device,
            surface,
            swap_chain,
            commands,
        }
    }

    pub fn frame<F>(&mut self, f: F)
    where
        F: Fn(&mut Frame),
    {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let chain_out = self.swap_chain.get_next_texture();
        for c in &self.commands {
            match c {
                Command::UpdateUniformBuffer(dst, src, size) => {
                    encoder.copy_buffer_to_buffer(&src, 0, &dst.wgpu, 0, *size as u32);
                }
            }
        }
        let mut frame = Frame {
            view: &chain_out.view,
            encoder,
        };
        f(&mut frame);

        self.device.get_queue().submit(&[frame.encoder.finish()]);
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

    // TODO: Should take a 'VecLike'.
    pub fn create_texture(&mut self, texels: Vec<u32>, w: u32, h: u32) -> Texture {
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

        let mut init_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        init_encoder.copy_buffer_to_texture(
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

        let init_command_buf = init_encoder.finish();
        self.device.get_queue().submit(&[init_command_buf]);
        Texture {
            wgpu: texture,
            view: texture_view,
        }
    }

    // TODO: Should take a 'VectorLike'.
    pub fn create_buffer<T>(&self, vertices: Vec<T>) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        VertexBuffer {
            wgpu: self
                .device
                .create_buffer_mapped(vertices.len(), wgpu::BufferUsageFlags::VERTEX)
                .fill_from_slice(&vertices),
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

    pub fn update_uniform_buffer<T>(&mut self, u: &'a UniformBuffer, buf: T)
    where
        T: 'static + Copy,
    {
        let tmp = self
            .device
            .create_buffer_mapped::<T>(
                1,
                wgpu::BufferUsageFlags::UNIFORM
                    | wgpu::BufferUsageFlags::TRANSFER_SRC
                    | wgpu::BufferUsageFlags::MAP_WRITE,
            )
            .fill_from_slice(&[buf]);

        self.commands.push(Command::UpdateUniformBuffer(
            u,
            tmp,
            std::mem::size_of::<T>(),
        ));
    }

    pub fn create_index(&self, indices: &[u16]) -> IndexBuffer {
        let index_buf = self
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsageFlags::INDEX)
            .fill_from_slice(&indices);
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

    pub fn create_bindings_layout(&self, slots: &[Slot]) -> BindingsLayout {
        let mut bindings = &mut self.bindings_layout_builder();

        for s in slots {
            bindings = bindings.binding(&s.stage, &s.binding);
        }
        bindings.build()
    }

    pub fn create_binding(&self, bs: &BindingSlots) -> Bindings {
        let mut bindings = &mut self.bindings_builder(bs.layout);

        for (i, s) in bs.slots.iter().enumerate() {
            match s {
                Binding::UniformBuffer(unif) => bindings = bindings.bind_uniforms(unif),
                Binding::Texture(tex) => bindings = bindings.bind_texture(tex),
                Binding::Sampler(sam) => bindings = bindings.bind_sampler(sam),
                Binding::Unbound() => panic!("binding slot {} is unbound", i),
            };
        }
        bindings.build()
    }

    pub fn bindings_layout_builder(&'a self) -> BindingsLayoutBuilder<'a> {
        BindingsLayoutBuilder {
            ctx: self,
            bindings: Vec::new(),
        }
    }

    pub fn bindings_builder(&'a self, layout: &'a BindingsLayout) -> BindingsBuilder<'a> {
        BindingsBuilder {
            ctx: self,
            layout,
            bindings: Vec::new(),
        }
    }

    pub fn create_pipeline(
        &self,
        binds: &BindingsLayout,
        vertex_layout: &VertexLayout,
        vs: &Shader,
        fs: &Shader,
    ) -> Pipeline {
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&binds.wgpu],
            });
        let vertex_attrs = vertex_layout.build();

        Pipeline {
            wgpu: self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    layout: &pipeline_layout,
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
                        color: wgpu::BlendDescriptor::REPLACE,
                        alpha: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWriteFlags::ALL,
                    }],
                    depth_stencil_state: None,
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[vertex_attrs.wgpu],
                    sample_count: 1,
                }),
        }
    }
}

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
