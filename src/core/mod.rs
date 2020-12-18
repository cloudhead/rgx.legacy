pub mod transform;

use bytemuck::Pod;
use wgpu::util::DeviceExt;

use raw_window_handle::HasRawWindowHandle;
use std::ops::Range;

// TODO: These shouldn't be re-exported from here.
pub use crate::color::{Bgra8, Rgba, Rgba8};
pub use crate::error::Error;
pub use crate::rect::Rect;

pub trait Renderable {
    fn buffer(&self, r: &Renderer) -> VertexBuffer;

    fn finish(self, r: &Renderer) -> VertexBuffer
    where
        Self: std::marker::Sized,
    {
        self.buffer(r)
    }
}

impl Rgba {
    fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

struct BufferDimensions {
    #[allow(dead_code)]
    width: u32,
    height: u32,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: u32, height: u32) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width as usize * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Draw
///////////////////////////////////////////////////////////////////////////////

pub trait Draw {
    fn draw<'a>(&'a self, binding: &'a BindingGroup, pass: &'a mut Pass<'a>);
}

///////////////////////////////////////////////////////////////////////////////
/// Shaders
///////////////////////////////////////////////////////////////////////////////

/// A GPU Shader.
#[derive(Debug)]
pub struct Shader {
    module: wgpu::ShaderModule,
}

/// Shader stage.
#[derive(Debug, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    fn to_wgpu(&self) -> wgpu::ShaderStage {
        match self {
            ShaderStage::Vertex => wgpu::ShaderStage::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStage::FRAGMENT,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Canvas
///////////////////////////////////////////////////////////////////////////////

pub trait Canvas {
    type Color;

    fn clear(&self, color: Self::Color, device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn fill(&self, buf: &[Self::Color], device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn transfer(
        &self,
        buf: &[Self::Color],
        w: u32,
        h: u32,
        r: Rect<i32>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    );
    fn blit(&self, from: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder);
}

///////////////////////////////////////////////////////////////////////////////
/// BindingGroup
///////////////////////////////////////////////////////////////////////////////

/// A group of bindings.
#[derive(Debug)]
pub struct BindingGroup {
    wgpu: wgpu::BindGroup,
    set_index: u32,
}

impl BindingGroup {
    fn new(set_index: u32, wgpu: wgpu::BindGroup) -> Self {
        Self { set_index, wgpu }
    }
}

/// The layout of a `BindingGroup`.
#[derive(Debug)]
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
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry;
}

///////////////////////////////////////////////////////////////////////////////
/// Uniforms
///////////////////////////////////////////////////////////////////////////////

/// A uniform buffer that can be bound in a 'BindingGroup'.
#[derive(Debug)]
pub struct UniformBuffer {
    wgpu: wgpu::Buffer,
    size: usize,
    count: usize,
}

impl Bind for UniformBuffer {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::Buffer(self.wgpu.slice(..)),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// ZBuffer
///////////////////////////////////////////////////////////////////////////////

/// Z-Depth buffer.
#[derive(Debug)]
pub struct ZBuffer {
    pub texture: Texture,
}

impl ZBuffer {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
}

///////////////////////////////////////////////////////////////////////////////
/// Framebuffer
///////////////////////////////////////////////////////////////////////////////

/// Off-screen framebuffer. Can be used as a render target in render passes.
#[derive(Debug)]
pub struct Framebuffer {
    pub texture: Texture,
    pub depth: ZBuffer,
}

impl Framebuffer {
    /// Size in pixels of the framebuffer.
    pub fn size(&self) -> usize {
        (self.texture.w * self.texture.h) as usize
    }

    /// Framebuffer width, in pixels.
    pub fn width(&self) -> u32 {
        self.texture.w
    }

    /// Framebuffer height, in pixels.
    pub fn height(&self) -> u32 {
        self.texture.h
    }
}

impl RenderTarget for Framebuffer {
    fn color_target(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    fn zdepth_target(&self) -> &wgpu::TextureView {
        &self.depth.texture.view
    }
}

impl Bind for Framebuffer {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.texture.view),
        }
    }
}

impl Canvas for Framebuffer {
    type Color = Bgra8;

    fn clear(&self, color: Bgra8, device: &mut Device, _encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(&self.texture, color, device);
        Texture::clear(&self.depth.texture, 0f32, device);
    }

    fn fill(&self, buf: &[Bgra8], device: &mut Device, _encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self.texture, buf, device);
    }

    fn transfer(
        &self,
        buf: &[Bgra8],
        w: u32,
        h: u32,
        rect: Rect<i32>,
        device: &mut Device,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self.texture, buf, w, h, rect, device);
    }

    fn blit(&self, from: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder) {
        Texture::blit(&self.texture, from, dst, encoder);
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Texturing
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Texture {
    wgpu: wgpu::Texture,
    view: wgpu::TextureView,
    extent: wgpu::Extent3d,
    format: wgpu::TextureFormat,

    pub w: u32,
    pub h: u32,
}

impl Texture {
    pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn rect(&self) -> Rect<f32> {
        Rect {
            x1: 0.0,
            y1: 0.0,
            x2: self.w as f32,
            y2: self.h as f32,
        }
    }

    fn clear<T>(texture: &Texture, value: T, device: &mut Device)
    where
        T: Clone,
    {
        let mut texels: Vec<T> = Vec::with_capacity(texture.w as usize * texture.h as usize);
        texels.resize(texture.w as usize * texture.h as usize, value);

        let (head, body, tail) = unsafe { texels.align_to::<Rgba8>() };
        assert!(head.is_empty());
        assert!(tail.is_empty());

        Self::fill(texture, body, device);
    }

    fn fill<T: 'static>(texture: &Texture, texels: &[T], device: &mut Device)
    where
        T: Clone + Copy + Pod,
    {
        assert_eq!(
            texels.len() as u32,
            texture.w * texture.h,
            "fatal: incorrect length for texel buffer"
        );

        device.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            bytemuck::cast_slice(&texels),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * texture.w,
                rows_per_image: texture.h,
            },
            wgpu::Extent3d {
                width: texture.w,
                height: texture.h,
                depth: 1,
            },
        );
    }

    fn transfer<T: 'static>(
        texture: &Texture,
        texels: &[T],
        width: u32,
        height: u32,
        rect: Rect<i32>,
        device: &mut Device,
    ) where
        T: Into<Rgba8> + Clone + Copy + Pod,
    {
        // Wgpu's coordinate system has a downwards pointing Y axis.
        let rect = rect.abs().flip_y();

        // The width and height of the transfer area.
        let tx_w = rect.width() as u32;
        let tx_h = rect.height() as u32;

        // The destination coordinate of the transfer, on the texture.
        // We have to invert the Y coordinate as explained above.
        let (dst_x, dst_y) = (rect.x1 as u32, texture.h - rect.y1 as u32);

        assert_eq!(
            texels.len() as u32,
            width * height,
            "fatal: incorrect length for texel buffer"
        );
        assert!(
            tx_w * tx_h <= texture.w * texture.h,
            "fatal: transfer size must be <= texture size"
        );

        let extent = wgpu::Extent3d {
            width: tx_w,
            height: tx_h,
            depth: 1,
        };

        device.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst_x,
                    y: dst_y,
                    z: 0,
                },
            },
            bytemuck::cast_slice(&texels),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * tx_w,
                rows_per_image: tx_h,
            },
            extent,
        );
    }

    fn blit(&self, src: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder) {
        assert!(
            (src.width() - dst.width()).abs() <= f32::EPSILON,
            "source and destination rectangles must be of the same size"
        );
        assert!(
            (src.height() - dst.height()).abs() <= f32::EPSILON,
            "source and destination rectangles must be of the same size"
        );

        encoder.copy_texture_to_texture(
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: src.x1 as u32,
                    y: src.y1 as u32,
                    z: 0,
                },
            },
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst.x1 as u32,
                    y: dst.y1 as u32,
                    z: 0,
                },
            },
            wgpu::Extent3d {
                width: src.width() as u32,
                height: src.height() as u32,
                depth: 1,
            },
        );
    }
}

impl Bind for Texture {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}

impl Canvas for Texture {
    type Color = Rgba8;

    fn fill(&self, buf: &[Rgba8], device: &mut Device, _encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self, buf, device);
    }

    fn clear(&self, color: Rgba8, device: &mut Device, _encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(&self, color, device);
    }

    fn transfer(
        &self,
        buf: &[Rgba8],
        w: u32,
        h: u32,
        rect: Rect<i32>,
        device: &mut Device,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self, buf, w, h, rect, device);
    }

    fn blit(&self, src: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder) {
        Texture::blit(&self, src, dst, encoder);
    }
}

impl From<Framebuffer> for Texture {
    fn from(fb: Framebuffer) -> Self {
        fb.texture
    }
}

#[derive(Debug)]
pub struct Sampler {
    wgpu: wgpu::Sampler,
}

impl Bind for Sampler {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
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

#[derive(Debug)]
pub struct VertexBuffer {
    pub size: u32,
    wgpu: wgpu::Buffer,
}

impl Draw for VertexBuffer {
    fn draw<'a>(&'a self, binding: &'a BindingGroup, pass: &'a mut Pass<'a>) {
        // TODO: If we attempt to draw more vertices than exist in the buffer, because
        // 'size' was guessed wrong, we get a wgpu error. We should somehow try to
        // get the pipeline layout to know here if the buffer we're trying to draw
        // is the right size. Another option is to create buffers from the pipeline,
        // so that we can check at creation time whether the data passed in matches
        // the format.
        pass.set_binding(binding, &[]);
        pass.draw_buffer(&self);
    }
}

#[derive(Debug)]
pub struct IndexBuffer {
    wgpu: wgpu::Buffer,
}

#[derive(Debug, Clone, Copy)]
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
#[derive(Default, Debug)]
pub struct VertexLayout {
    wgpu_attrs: Vec<wgpu::VertexAttributeDescriptor>,
    size: usize,
}

impl VertexLayout {
    pub fn from(formats: &[VertexFormat]) -> Self {
        let mut vl = Self::default();
        for vf in formats {
            vl.wgpu_attrs.push(wgpu::VertexAttributeDescriptor {
                shader_location: vl.wgpu_attrs.len() as u32,
                offset: vl.size as wgpu::BufferAddress,
                format: vf.to_wgpu(),
            });
            vl.size += vf.bytesize();
        }
        vl
    }

    fn to_wgpu(&self) -> wgpu::VertexBufferDescriptor {
        wgpu::VertexBufferDescriptor {
            stride: self.size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: self.wgpu_attrs.as_slice(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline Bindings
///////////////////////////////////////////////////////////////////////////////

/// A binding type.
#[derive(Debug)]
pub enum BindingType {
    UniformBuffer,
    UniformBufferDynamic,
    Sampler,
    SampledTexture,
}

impl BindingType {
    fn to_wgpu(&self) -> wgpu::BindingType {
        match self {
            // XXX: Binding size should be non-zero?
            BindingType::UniformBuffer => wgpu::BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            BindingType::UniformBufferDynamic => wgpu::BindingType::UniformBuffer {
                dynamic: true,
                min_binding_size: None,
            },
            BindingType::SampledTexture => wgpu::BindingType::SampledTexture {
                multisampled: false,
                dimension: wgpu::TextureViewDimension::D2,
                component_type: wgpu::TextureComponentType::Float,
            },
            BindingType::Sampler => wgpu::BindingType::Sampler { comparison: false },
        }
    }
}

#[derive(Debug)]
pub struct Binding {
    pub binding: BindingType,
    pub stage: ShaderStage,
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Pipeline {
    wgpu: wgpu::RenderPipeline,

    pub layout: PipelineLayout,
    pub vertex_layout: VertexLayout,
}

impl Pipeline {
    pub fn apply<'a>(&'a self, pass: &mut Pass<'a>) {
        pass.wgpu.set_pipeline(&self.wgpu);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blending {
    src_factor: BlendFactor,
    dst_factor: BlendFactor,
    operation: BlendOp,
}

impl Blending {
    pub fn new(src_factor: BlendFactor, dst_factor: BlendFactor, operation: BlendOp) -> Self {
        Blending {
            src_factor,
            dst_factor,
            operation,
        }
    }

    pub fn constant() -> Self {
        Blending {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            operation: BlendOp::Add,
        }
    }

    fn to_wgpu(&self) -> (wgpu::BlendFactor, wgpu::BlendFactor, wgpu::BlendOperation) {
        (
            self.src_factor.to_wgpu(),
            self.dst_factor.to_wgpu(),
            self.operation.to_wgpu(),
        )
    }
}

impl Default for Blending {
    fn default() -> Self {
        Blending {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOp::Add,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendFactor {
    One,
    Zero,
    SrcAlpha,
    OneMinusSrcAlpha,
}

impl BlendFactor {
    fn to_wgpu(&self) -> wgpu::BlendFactor {
        match self {
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendOp {
    Add,
}

impl BlendOp {
    fn to_wgpu(&self) -> wgpu::BlendOperation {
        match self {
            BlendOp::Add => wgpu::BlendOperation::Add,
        }
    }
}

#[derive(Debug)]
pub struct Set<'a>(pub &'a [Binding]);

#[derive(Debug)]
pub struct PipelineLayout {
    pub sets: Vec<BindingGroupLayout>,
}

pub trait AbstractPipeline<'a> {
    type PrepareContext;
    type Uniforms: Copy + Pod + 'static;

    fn description() -> PipelineDescription<'a>;
    fn setup(pip: Pipeline, dev: &Device) -> Self;
    fn apply(&'a self, pass: &mut Pass<'a>);
    fn prepare(
        &'a self,
        t: Self::PrepareContext,
    ) -> Option<(&'a UniformBuffer, Vec<Self::Uniforms>)>;
}

#[derive(Debug)]
pub struct PipelineDescription<'a> {
    pub vertex_layout: &'a [VertexFormat],
    pub pipeline_layout: &'a [Set<'a>],
    pub vertex_shader: &'static [u8],
    pub fragment_shader: &'static [u8],
}

///////////////////////////////////////////////////////////////////////////////
/// Frame
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Frame {
    encoder: wgpu::CommandEncoder,
}

impl Frame {
    pub fn new(encoder: wgpu::CommandEncoder) -> Self {
        Self { encoder }
    }

    pub fn pass<'a, T: RenderTarget>(&'a mut self, op: PassOp, view: &'a T) -> Pass<'a> {
        Pass::begin(
            &mut self.encoder,
            &view.color_target(),
            &view.zdepth_target(),
            op,
        )
    }

    pub fn copy(&mut self, src: &UniformBuffer, dst: &UniformBuffer) {
        self.encoder.copy_buffer_to_buffer(
            &src.wgpu,
            0,
            &dst.wgpu,
            0,
            (src.size * src.count) as wgpu::BufferAddress,
        );
    }

    pub fn encoder(&self) -> &wgpu::CommandEncoder {
        &self.encoder
    }

    pub fn encoder_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pass
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Pass<'a> {
    wgpu: wgpu::RenderPass<'a>,
}

impl<'a, 'b> Pass<'a> {
    pub fn begin(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
        depth: &'a wgpu::TextureView,
        op: PassOp,
    ) -> Pass<'a> {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                ops: wgpu::Operations {
                    load: op.to_wgpu(),
                    store: true,
                },
                resolve_target: None,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        });
        Pass { wgpu: pass }
    }
    pub fn set_pipeline<T>(&mut self, pipeline: &'a T)
    where
        T: AbstractPipeline<'a>,
    {
        pipeline.apply(self);
    }
    pub fn set_binding(&mut self, group: &'a BindingGroup, offsets: &[u32]) {
        self.wgpu
            .set_bind_group(group.set_index, &group.wgpu, offsets);
    }
    pub fn set_index_buffer(&mut self, index_buf: &'a IndexBuffer) {
        self.wgpu.set_index_buffer(index_buf.wgpu.slice(..))
    }
    pub fn set_vertex_buffer(&mut self, vertex_buf: &'a VertexBuffer) {
        self.wgpu.set_vertex_buffer(0, vertex_buf.wgpu.slice(..))
    }
    pub fn draw<T: Draw>(&'a mut self, drawable: &'a T, binding: &'a BindingGroup) {
        drawable.draw(binding, self);
    }
    pub fn draw_buffer(&mut self, buf: &'a VertexBuffer) {
        self.set_vertex_buffer(buf);
        self.wgpu.draw(0..buf.size, 0..1);
    }
    pub fn draw_buffer_range(&mut self, buf: &'a VertexBuffer, range: Range<u32>) {
        self.set_vertex_buffer(buf);
        self.wgpu.draw(range, 0..1);
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw_indexed(indices, 0, instances)
    }
}

#[derive(Debug)]
pub enum PassOp {
    Clear(Rgba),
    Load(),
}

impl PassOp {
    fn to_wgpu(&self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            PassOp::Clear(color) => wgpu::LoadOp::Clear(color.to_wgpu()),
            PassOp::Load() => wgpu::LoadOp::Load,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// SwapChain & RenderTarget
///////////////////////////////////////////////////////////////////////////////

/// Can be rendered to in a pass.
pub trait RenderTarget {
    /// Color component.
    fn color_target(&self) -> &wgpu::TextureView;
    /// Depth component.
    fn zdepth_target(&self) -> &wgpu::TextureView;
}

#[derive(Debug)]
pub struct SwapChainTexture<'a> {
    pub width: u32,
    pub height: u32,

    wgpu: wgpu::SwapChainTexture,
    depth: &'a ZBuffer,
}

impl RenderTarget for SwapChainTexture<'_> {
    fn color_target(&self) -> &wgpu::TextureView {
        &self.wgpu.view
    }

    fn zdepth_target(&self) -> &wgpu::TextureView {
        &self.depth.texture.view
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    Vsync,
    NoVsync,
}

impl PresentMode {
    fn to_wgpu(&self) -> wgpu::PresentMode {
        match self {
            PresentMode::Vsync => wgpu::PresentMode::Fifo, // TODO: Use `Mailbox`
            PresentMode::NoVsync => wgpu::PresentMode::Immediate,
        }
    }
}

impl Default for PresentMode {
    fn default() -> Self {
        PresentMode::Vsync
    }
}

/// A handle to a swap chain.
///
/// A `SwapChain` represents the image or series of images that will be presented to a [`Renderer`].
/// A `SwapChain` may be created with [`Renderer::swap_chain`].
#[derive(Debug)]
pub struct SwapChain {
    pub width: u32,
    pub height: u32,

    depth: ZBuffer,
    wgpu: wgpu::SwapChain,
}

impl SwapChain {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    /// Convenience method to retrieve `(width, height)`
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the next texture to be presented by the swapchain for drawing.
    ///
    /// When the [`SwapChainTexture`] returned by this method is dropped, the
    /// swapchain will present the texture to the associated [`Renderer`].
    pub fn next(&mut self) -> Result<SwapChainTexture, wgpu::SwapChainError> {
        let frame = self.wgpu.get_current_frame()?;

        Ok(SwapChainTexture {
            depth: &self.depth,
            wgpu: frame.output,
            width: self.width,
            height: self.height,
        })
    }

    /// Get the texture format in use
    pub fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }

    fn descriptor(width: u32, height: u32, mode: PresentMode) -> wgpu::SwapChainDescriptor {
        wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: Self::FORMAT,
            present_mode: mode.to_wgpu(),
            width,
            height,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Renderer
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Renderer {
    pub device: Device,
}

impl Renderer {
    pub async fn new<W: HasRawWindowHandle>(window: &W) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(Error::NoAdaptersFound)?;

        Ok(Self {
            device: Device::new(&adapter, surface).await,
        })
    }

    pub fn swap_chain(&self, w: u32, h: u32, mode: PresentMode) -> SwapChain {
        SwapChain {
            depth: self.device.create_zbuffer(w, h),
            wgpu: self.device.create_swap_chain(w, h, mode),
            width: w,
            height: h,
        }
    }

    pub fn texture(&self, w: u32, h: u32) -> Texture {
        self.device.create_texture(w, h)
    }

    pub fn framebuffer(&self, w: u32, h: u32) -> Framebuffer {
        self.device.create_framebuffer(w, h)
    }

    pub fn zbuffer(&self, w: u32, h: u32) -> ZBuffer {
        self.device.create_zbuffer(w, h)
    }

    pub fn vertex_buffer<T: Pod>(&self, verts: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_buffer(verts)
    }

    pub fn uniform_buffer<T: Pod>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_uniform_buffer(buf)
    }

    pub fn binding_group(&self, layout: &BindingGroupLayout, binds: &[&dyn Bind]) -> BindingGroup {
        self.device.create_binding_group(layout, binds)
    }

    pub fn sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        self.device.create_sampler(min_filter, mag_filter)
    }

    pub fn pipeline<T>(&self, blending: Blending) -> T
    where
        T: AbstractPipeline<'static>,
    {
        let desc = T::description();
        let pip_layout = self.device.create_pipeline_layout(desc.pipeline_layout);
        let vertex_layout = VertexLayout::from(desc.vertex_layout);
        let vs =
            self.device
                .create_shader("vertex shader", desc.vertex_shader, ShaderStage::Vertex);
        let fs = self.device.create_shader(
            "fragment shader",
            desc.fragment_shader,
            ShaderStage::Fragment,
        );

        T::setup(
            self.device
                .create_pipeline(pip_layout, vertex_layout, blending, &vs, &fs),
            &self.device,
        )
    }

    pub fn read(&mut self, fb: &Framebuffer) -> Vec<Bgra8> {
        let dimensions = BufferDimensions::new(fb.texture.w, fb.texture.h);
        let bytes_per_row = dimensions.padded_bytes_per_row;
        let bytes_total = bytes_per_row * dimensions.height as usize;

        let dst = self.device.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: bytes_total as u64,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: true,
        });

        let command_buffer = {
            let mut encoder = self.device.create_command_encoder();
            encoder.copy_texture_to_buffer(
                wgpu::TextureCopyView {
                    texture: &fb.texture.wgpu,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::BufferCopyView {
                    buffer: &dst,
                    layout: wgpu::TextureDataLayout {
                        offset: 0,
                        bytes_per_row: bytes_per_row as u32,
                        rows_per_image: dimensions.height,
                    },
                },
                fb.texture.extent,
            );
            encoder.finish()
        };

        let mut buffer: Vec<u8> = Vec::with_capacity(fb.size() * std::mem::size_of::<u32>());
        {
            let view = dst.slice(..).get_mapped_range();
            for row in view.chunks(bytes_per_row) {
                buffer.extend_from_slice(&row[..dimensions.unpadded_bytes_per_row]);
            }
        }
        dst.unmap();

        self.device.submit(Some(command_buffer));

        let (head, body, tail) = unsafe { buffer.align_to::<Bgra8>() };
        if !(head.is_empty() && tail.is_empty()) {
            panic!("Renderer::read: framebuffer is not a valid Bgra8 buffer");
        }
        body.to_owned()
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn update_pipeline<'a, T>(&mut self, pip: &'a T, p: T::PrepareContext, f: &mut Frame)
    where
        T: AbstractPipeline<'a>,
    {
        if let Some((buf, unifs)) = pip.prepare(p) {
            self.device
                .update_uniform_buffer::<T::Uniforms>(unifs.as_slice(), buf, &mut f.encoder);
        }
    }

    pub fn frame(&mut self) -> Frame {
        let encoder = self.device.create_command_encoder();
        Frame::new(encoder)
    }

    pub fn present(&mut self, frame: Frame) {
        self.device.submit(Some(frame.encoder.finish()));
    }

    pub fn submit<T: Copy>(&mut self, commands: &[Op<T>]) {
        let mut encoder = self.device.create_command_encoder();
        for c in commands.iter() {
            c.encode(&mut self.device, &mut encoder);
        }
        self.device.submit(Some(encoder.finish()));
    }
}

pub enum Op<'a, T> {
    Clear(&'a dyn Canvas<Color = T>, T),
    Fill(&'a dyn Canvas<Color = T>, &'a [T]),
    Transfer(&'a dyn Canvas<Color = T>, &'a [T], u32, u32, Rect<i32>),
    Blit(&'a dyn Canvas<Color = T>, Rect<f32>, Rect<f32>),
}

impl<'a, T> Op<'a, T>
where
    T: Copy,
{
    fn encode(&self, dev: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        match *self {
            Op::Clear(f, color) => {
                f.clear(color, dev, encoder);
            }
            Op::Fill(f, buf) => {
                f.fill(buf, dev, encoder);
            }
            Op::Transfer(f, buf, w, h, rect) => {
                f.transfer(buf, w, h, rect, dev, encoder);
            }
            Op::Blit(f, src, dst) => {
                f.blit(src, dst, encoder);
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Device
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
}

impl Device {
    pub async fn new(adapter: &wgpu::Adapter, surface: wgpu::Surface) -> Self {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        Self {
            device,
            queue,
            surface,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.device
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    pub fn create_swap_chain(&self, w: u32, h: u32, mode: PresentMode) -> wgpu::SwapChain {
        let desc = SwapChain::descriptor(w, h, mode);
        self.device.create_swap_chain(&self.surface, &desc)
    }

    pub fn create_pipeline_layout(&self, ss: &[Set]) -> PipelineLayout {
        let mut sets = Vec::new();
        for (i, s) in ss.iter().enumerate() {
            sets.push(self.create_binding_group_layout(i as u32, s.0))
        }
        PipelineLayout { sets }
    }

    pub fn create_shader(&self, _name: &str, source: &[u8], _stage: ShaderStage) -> Shader {
        let spv = wgpu::util::make_spirv(source);
        Shader {
            module: self.device.create_shader_module(spv),
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    pub fn create_texture(&self, w: u32, h: u32) -> Texture {
        let format = Texture::COLOR_FORMAT;
        let texture_extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            wgpu: texture,
            view: texture_view,
            extent: texture_extent,
            format,
            w,
            h,
        }
    }

    pub fn create_framebuffer(&self, w: u32, h: u32) -> Framebuffer {
        let format = SwapChain::FORMAT;
        let extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Framebuffer {
            texture: Texture {
                wgpu: texture,
                view,
                extent,
                format,
                w,
                h,
            },
            depth: self.create_zbuffer(w, h),
        }
    }

    pub fn create_zbuffer(&self, w: u32, h: u32) -> ZBuffer {
        let format = ZBuffer::FORMAT;
        let extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let wgpu = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let view = wgpu.create_view(&wgpu::TextureViewDescriptor::default());

        ZBuffer {
            texture: Texture {
                wgpu,
                extent,
                view,
                format,
                w,
                h,
            },
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
                label: None,
                layout: &layout.wgpu,
                entries: bindings.as_slice(),
            }),
        )
    }

    pub fn create_buffer<T: Pod>(&self, vertices: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        VertexBuffer {
            wgpu: self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsage::VERTEX,
                }),
            size: vertices.len() as u32,
        }
    }

    pub fn create_uniform_buffer<T: Pod>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        UniformBuffer {
            size: std::mem::size_of::<T>(),
            count: buf.len(),
            wgpu: self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&buf),
                    usage: wgpu::BufferUsage::UNIFORM
                        | wgpu::BufferUsage::COPY_DST
                        | wgpu::BufferUsage::COPY_SRC,
                }),
        }
    }

    pub fn create_index(&self, indices: &[u16]) -> IndexBuffer {
        let index_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsage::INDEX,
            });
        IndexBuffer { wgpu: index_buf }
    }

    pub fn create_sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        Sampler {
            wgpu: self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: mag_filter.to_wgpu(),
                min_filter: min_filter.to_wgpu(),
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: None,
            }),
        }
    }

    pub fn create_binding_group_layout(&self, index: u32, slots: &[Binding]) -> BindingGroupLayout {
        let mut bindings = Vec::new();

        for s in slots {
            bindings.push(wgpu::BindGroupLayoutEntry {
                binding: bindings.len() as u32,
                visibility: s.stage.to_wgpu(),
                ty: s.binding.to_wgpu(),
                count: None,
            });
        }
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: bindings.as_slice(),
            });
        BindingGroupLayout::new(index, layout, bindings.len())
    }

    pub fn update_uniform_buffer<T: Pod + Copy + 'static>(
        &self,
        slice: &[T],
        buf: &UniformBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let src = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&slice),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
            });

        encoder.copy_buffer_to_buffer(
            &src,
            0,
            &buf.wgpu,
            0,
            (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress,
        );
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(&mut self, cmds: I) {
        self.queue.submit(cmds);
    }

    // PRIVATE API ////////////////////////////////////////////////////////////

    fn create_pipeline(
        &self,
        pipeline_layout: PipelineLayout,
        vertex_layout: VertexLayout,
        blending: Blending,
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
                label: None,
                bind_group_layouts: sets.as_slice(),
                push_constant_ranges: &[],
            });

        let (src_factor, dst_factor, operation) = blending.to_wgpu();

        let wgpu = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(layout),
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs.module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs.module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    clamp_depth: self
                        .device
                        .features()
                        .contains(wgpu::Features::DEPTH_CLAMPING),
                    ..Default::default()
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: SwapChain::FORMAT,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor,
                        dst_factor,
                        operation,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor,
                        dst_factor,
                        operation,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                    format: ZBuffer::FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilStateDescriptor {
                        front: wgpu::StencilStateFaceDescriptor::IGNORE,
                        back: wgpu::StencilStateFaceDescriptor::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                }),
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    // index_format: None,
                    vertex_buffers: &[vertex_attrs],
                    // vertex_buffers: &[],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Pipeline {
            layout: pipeline_layout,
            vertex_layout,
            wgpu,
        }
    }
}
