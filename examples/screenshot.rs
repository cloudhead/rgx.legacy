#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use self::kit::shape2d;
use self::kit::shape2d::*;
use rgx::core;
use rgx::core::*;
use rgx::kit;
use rgx::math::*;

use image::png::PNGEncoder;
use image::ColorType;

use std::fs::File;

use winit::{event_loop::EventLoop, window::Window};

pub struct Framebuffer {
    target: core::Framebuffer,
    vertices: core::VertexBuffer,
}

impl Framebuffer {
    fn new(w: u32, h: u32, r: &core::Renderer) -> Self {
        #[rustfmt::skip]
        let vertices: &[[f32; 4]] = &[
            [-1.0, -1.0, 0.0, 1.0],
            [ 1.0, -1.0, 1.0, 1.0],
            [ 1.0,  1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
            [-1.0,  1.0, 0.0, 0.0],
            [ 1.0,  1.0, 1.0, 0.0],
        ];

        Self {
            target: r.framebuffer(w, h),
            vertices: r.vertex_buffer(vertices),
        }
    }
}

pub struct FramebufferPipeline {
    pipeline: core::Pipeline,
    bindings: core::BindingGroup,
    buf: core::UniformBuffer,
}

impl<'a> core::AbstractPipeline<'a> for FramebufferPipeline {
    type PrepareContext = core::Rgba;
    type Uniforms = core::Rgba;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[core::VertexFormat::Float2, core::VertexFormat::Float2],
            pipeline_layout: &[
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[
                    Binding {
                        binding: BindingType::SampledTexture,
                        stage: ShaderStage::Fragment,
                    },
                    Binding {
                        binding: BindingType::Sampler,
                        stage: ShaderStage::Fragment,
                    },
                ]),
            ],
            // TODO: Use `env("CARGO_MANIFEST_DIR")`
            vertex_shader: include_bytes!("data/framebuffer.vert.spv"),
            fragment_shader: include_bytes!("data/framebuffer.frag.spv"),
        }
    }

    fn setup(pipeline: core::Pipeline, dev: &core::Device) -> Self {
        let buf = dev.create_uniform_buffer(&[core::Rgba::TRANSPARENT]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        FramebufferPipeline {
            pipeline,
            buf,
            bindings,
        }
    }

    fn apply(&'a self, pass: &mut core::Pass<'a>) {
        self.pipeline.apply(pass);
        pass.set_binding(&self.bindings, &[]);
    }

    fn prepare(&'a self, color: core::Rgba) -> Option<(&'a core::UniformBuffer, Vec<core::Rgba>)> {
        Some((&self.buf, vec![color]))
    }
}

impl FramebufferPipeline {
    pub fn binding(
        &self,
        renderer: &core::Renderer,
        framebuffer: &Framebuffer,
        sampler: &core::Sampler,
    ) -> core::BindingGroup {
        renderer.device.create_binding_group(
            &self.pipeline.layout.sets[1],
            &[&framebuffer.target, sampler],
        )
    }
}

fn main() -> Result<(), std::io::Error> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = futures::executor::block_on(Renderer::new(&window))?;
    let size = window.inner_size();

    let (sw, sh) = (size.width as u32, size.height as u32);
    let framebuffer = Framebuffer::new(sw, sh, &r);

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);
    let mut textures = r.swap_chain(sw, sh, PresentMode::default());

    let offscreen: kit::shape2d::Pipeline = r.pipeline(Blending::default());
    let onscreen: FramebufferPipeline = r.pipeline(Blending::default());
    let onscreen_binding = onscreen.binding(&r, &framebuffer, &sampler);

    let buffer = shape2d::Batch::singleton(
        Shape::circle(
            Point2::new(sw as f32 / 2., sh as f32 / 2.),
            sh as f32 / 2.0,
            128,
        )
        .stroke(3.0, Rgba::new(1.0, 0.0, 1.0, 1.0)),
    )
    .finish(&r);

    ///////////////////////////////////////////////////////////////////////////
    // Create frame & on-screen output texture
    ///////////////////////////////////////////////////////////////////////////

    let mut frame = r.frame();
    let out = textures.next().unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Update pipeline
    ///////////////////////////////////////////////////////////////////////////

    r.update_pipeline(
        &offscreen,
        kit::ortho(out.width, out.height, kit::Origin::BottomLeft),
        &mut frame,
    );
    r.update_pipeline(&onscreen, Rgba::TRANSPARENT, &mut frame);

    ///////////////////////////////////////////////////////////////////////////
    // Draw frame
    ///////////////////////////////////////////////////////////////////////////

    {
        let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &framebuffer.target);
        pass.set_pipeline(&offscreen);
        pass.draw_buffer(&buffer);
    }

    {
        let mut pass = frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);
        pass.set_pipeline(&onscreen);
        pass.set_binding(&onscreen_binding, &[]);
        pass.draw_buffer(&framebuffer.vertices);
    }

    // Present frame first, so that we can read it below.
    r.present(frame);

    ///////////////////////////////////////////////////////////////////////////
    // Read the framebuffer into host memory and write it to an image file
    ///////////////////////////////////////////////////////////////////////////

    let w = framebuffer.target.texture.w;
    let h = framebuffer.target.texture.h;

    let data = r.read(&framebuffer.target);
    let file = File::create("screenshot.png").unwrap();
    let png = PNGEncoder::new(file);
    let (_, bytes, _) = unsafe { data.align_to::<u8>() };

    // Nb. The blue and red channel are swapped, since our
    // framebuffer data is in BGRA format.
    png.encode(bytes, w, h, ColorType::BGRA(8)).unwrap();

    Ok(())
}
