#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core;
use rgx::core::*;
use rgx::kit;
use rgx::kit::sprite2d;
use rgx::kit::*;

use image::ImageDecoder;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

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

    fn apply<'b>(&'b self, pass: &mut core::Pass<'b>) {
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
    let offscreen: kit::sprite2d::Pipeline = r.pipeline(Blending::default());
    let onscreen: FramebufferPipeline = r.pipeline(Blending::default());
    let framebuffer = Framebuffer::new(sw, sh, &r);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);

    let (texture, pixels) = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (w, h) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();
        let pixels = Rgba8::align(&pixels);

        (r.texture(w as u32, h as u32), pixels.to_owned())
    };

    let offscreen_binding = offscreen.binding(&r, &texture, &sampler); // Texture binding
    let onscreen_binding = onscreen.binding(&r, &framebuffer, &sampler);

    let w = 50.0;
    let rect = Rect::new(w * 1.0, 0.0, w * 2.0, texture.h as f32);
    let batch = sprite2d::Batch::singleton(
        texture.w,
        texture.h,
        rect,
        Rect::origin(sw as f32, sh as f32),
        ZDepth::ZERO,
        Rgba::TRANSPARENT,
        1.0,
        Repeat::default(),
    );
    let buffer = batch.finish(&r);

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    r.submit(&[Op::Fill(&texture, pixels.as_slice())]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let mut textures = r.swap_chain(sw, sh, PresentMode::default());

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                let physical = size;
                let (w, h) = (physical.width as u32, physical.height as u32);

                textures = r.swap_chain(w, h, PresentMode::default());
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            *control_flow = ControlFlow::Wait;

            ///////////////////////////////////////////////////////////////////////////
            // Create frame
            ///////////////////////////////////////////////////////////////////////////

            let mut frame = r.frame();

            ///////////////////////////////////////////////////////////////////////////
            // Prepare pipeline
            ///////////////////////////////////////////////////////////////////////////

            let out = textures.next().unwrap();

            r.update_pipeline(
                &offscreen,
                kit::ortho(out.width, out.height, Default::default()),
                &mut frame,
            );
            r.update_pipeline(&onscreen, Rgba::new(0.2, 0.2, 0.0, 1.0), &mut frame);

            ///////////////////////////////////////////////////////////////////////////
            // Draw frame
            ///////////////////////////////////////////////////////////////////////////

            {
                let mut pass = frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &framebuffer.target);
                pass.set_pipeline(&offscreen);
                pass.set_binding(&offscreen_binding, &[]);
                pass.draw_buffer(&buffer);
            }

            {
                let mut pass = frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);
                pass.set_pipeline(&onscreen);
                pass.set_binding(&onscreen_binding, &[]);
                pass.draw_buffer(&framebuffer.vertices);
            }

            r.present(frame);
        }
        _ => {}
    });
}
