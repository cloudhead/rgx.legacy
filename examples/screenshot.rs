#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use self::kit::shape2d::*;
use rgx::core;
use rgx::core::*;
use rgx::kit;

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2};

use image::png::PNGEncoder;
use image::ColorType;

use std::fs::File;

use wgpu::winit::{EventsLoop, Window};

pub struct Framebuffer {
    target: core::Framebuffer,
    vertices: core::VertexBuffer,
}

impl Framebuffer {
    fn new(w: u32, h: u32, r: &core::Renderer) -> Self {
        #[rustfmt::skip]
        let vertices: &[(f32, f32, f32, f32)] = &[
            (-1.0, -1.0, 0.0, 1.0),
            ( 1.0, -1.0, 1.0, 1.0),
            ( 1.0,  1.0, 1.0, 0.0),
            (-1.0, -1.0, 0.0, 1.0),
            (-1.0,  1.0, 0.0, 0.0),
            ( 1.0,  1.0, 1.0, 0.0),
        ];

        Self {
            target: r.framebuffer(w, h),
            vertices: r.vertexbuffer(vertices),
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
            vertex_shader: include_str!("data/framebuffer.vert"),
            fragment_shader: include_str!("data/framebuffer.frag"),
        }
    }

    fn setup(pipeline: core::Pipeline, dev: &core::Device, _w: u32, _h: u32) -> Self {
        let buf = dev.create_uniform_buffer(&[core::Rgba::TRANSPARENT]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        FramebufferPipeline {
            pipeline,
            buf,
            bindings,
        }
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_binding(&self.bindings, &[0]);
    }

    fn prepare(&'a self, color: core::Rgba) -> Option<(&'a core::UniformBuffer, Vec<core::Rgba>)> {
        Some((&self.buf, vec![color]))
    }

    fn resize(&mut self, _w: u32, _h: u32) {}
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

fn main() {
    env_logger::init();

    let events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = Renderer::new(&window);

    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let (sw, sh) = (size.width as u32, size.height as u32);
    let framebuffer = Framebuffer::new(sw, sh, &r);

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);

    let offscreen: kit::shape2d::Pipeline = r.pipeline(sw, sh);
    let onscreen: FramebufferPipeline = r.pipeline(sw, sh);
    let onscreen_binding = onscreen.binding(&r, &framebuffer, &sampler);

    let sv = ShapeView::singleton(Shape::Circle(
        Vector2::new(sw as f32 / 2., sh as f32 / 2.),
        sh as f32 / 2.0,
        128,
        Stroke::new(3.0, Rgba::new(1.0, 0.0, 1.0, 1.0)),
        Fill::Empty(),
    ));
    let buffer = sv.finish(&r);

    ///////////////////////////////////////////////////////////////////////////
    // Create frame
    ///////////////////////////////////////////////////////////////////////////

    let mut frame = r.frame();

    ///////////////////////////////////////////////////////////////////////////
    // Prepare pipeline
    ///////////////////////////////////////////////////////////////////////////

    frame.prepare(&offscreen, Matrix4::identity());
    frame.prepare(&onscreen, Rgba::TRANSPARENT);

    ///////////////////////////////////////////////////////////////////////////
    // Draw frame
    ///////////////////////////////////////////////////////////////////////////

    {
        let pass = &mut frame.offscreen_pass(PassOp::Clear(Rgba::TRANSPARENT), &framebuffer.target);
        pass.set_pipeline(&offscreen);
        pass.set_vertex_buffer(&buffer);
        pass.draw_buffer(0..buffer.size, 0..1);
    }

    {
        let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT));
        pass.set_pipeline(&onscreen);
        pass.draw(&framebuffer.vertices, &onscreen_binding);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Read the framebuffer into host memory and write it to an image file
    ///////////////////////////////////////////////////////////////////////////

    let w = framebuffer.target.texture.w;
    let h = framebuffer.target.texture.h;

    frame.read(&framebuffer.target, move |data| {
        let file = File::create("screenshot.png").unwrap();
        let png = PNGEncoder::new(file);

        png.encode(data, w, h, ColorType::RGBA(8)).unwrap();
    });
}
