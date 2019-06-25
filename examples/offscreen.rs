#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core;
use rgx::core::*;
use rgx::kit;
use rgx::kit::sprite2d::TextureView;
use rgx::kit::*;

use cgmath::prelude::*;
use cgmath::Matrix4;

use image::ImageDecoder;

use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

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
            target: r.framebuffer(&[], w, h),
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
        pass.apply_pipeline(&self.pipeline);
        pass.apply_binding(&self.bindings, &[0]);
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

    let mut events_loop = EventsLoop::new();
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
    let mut offscreen: kit::sprite2d::Pipeline = r.pipeline(sw, sh);
    let mut onscreen: FramebufferPipeline = r.pipeline(sw, sh);
    let framebuffer = Framebuffer::new(sw, sh, &r);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);

    let texture = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (w, h) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();

        r.texture(pixels.as_slice(), w as u32, h as u32)
    };

    let offscreen_binding = offscreen.binding(&r, &texture, &sampler); // Texture binding
    let onscreen_binding = onscreen.binding(&r, &framebuffer, &sampler);

    let w = 50.0;
    let rect = Rect::new(w * 1.0, 0.0, w * 2.0, texture.h as f32);
    let tv = TextureView::singleton(
        texture.w,
        texture.h,
        rect,
        Rect::origin(sw as f32, sh as f32),
        Rgba::TRANSPARENT,
        1.0,
        Repeat::default(),
    );
    let buffer = tv.finish(&r);

    let mut running = true;

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    r.prepare(&[&texture]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        running = false;
                    }
                    WindowEvent::CloseRequested => {
                        running = false;
                    }
                    WindowEvent::Resized(size) => {
                        let physical = size.to_physical(window.get_hidpi_factor());
                        let (w, h) = (physical.width as u32, physical.height as u32);

                        offscreen.resize(w, h);
                        onscreen.resize(w, h);
                        r.resize(w, h);
                    }
                    _ => {}
                }
            }
        });

        ///////////////////////////////////////////////////////////////////////////
        // Create frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = r.frame();

        ///////////////////////////////////////////////////////////////////////////
        // Prepare pipeline
        ///////////////////////////////////////////////////////////////////////////

        frame.prepare(&offscreen, Matrix4::identity());
        frame.prepare(&onscreen, Rgba::new(0.2, 0.2, 0.0, 1.0));

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        {
            let pass = &mut frame.offscreen_pass(Rgba::TRANSPARENT, &framebuffer.target);
            pass.apply_pipeline(&offscreen);
            pass.draw(&buffer, &offscreen_binding);
        }

        {
            let pass = &mut frame.pass(Rgba::TRANSPARENT);
            pass.apply_pipeline(&onscreen);
            pass.draw(&framebuffer.vertices, &onscreen_binding);
        }
    }
}
