#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate rgx;

use rgx::core::*;
use rgx::kit::*;

use cgmath::{Matrix4, Vector3};

use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup rgx context
    ///////////////////////////////////////////////////////////////////////////

    let kit = Kit::new(&window);
    let mut ctx = kit.ctx;

    ///////////////////////////////////////////////////////////////////////////
    // Setup shaders
    ///////////////////////////////////////////////////////////////////////////

    let vs = ctx.create_shader(
        "shader.vert",
        include_str!("data/shader.vert"),
        ShaderStage::Vertex,
    );

    let fs = ctx.create_shader(
        "shader.frag",
        include_str!("data/shader.frag"),
        ShaderStage::Fragment,
    );

    ///////////////////////////////////////////////////////////////////////////
    // Setup shader bindings layout
    ///////////////////////////////////////////////////////////////////////////

    let vertex_layout = VertexLayout::from(&[
        VertexFormat::Float2,
        VertexFormat::Float4,
        VertexFormat::Float2,
    ]);

    let uniforms_layout = ctx.create_uniforms_layout(&[
        Slot {
            binding: BindingType::UniformBuffer,
            stage: ShaderStage::Vertex,
        },
        Slot {
            binding: BindingType::SampledTexture,
            stage: ShaderStage::Fragment,
        },
        Slot {
            binding: BindingType::Sampler,
            stage: ShaderStage::Fragment,
        },
    ]);

    ///////////////////////////////////////////////////////////////////////////
    // Setup render pipeline
    ///////////////////////////////////////////////////////////////////////////

    let pipeline = ctx.create_pipeline(&uniforms_layout, &vertex_layout, &vs, &fs);

    ///////////////////////////////////////////////////////////////////////////
    // Setup texture & sampler
    ///////////////////////////////////////////////////////////////////////////

    #[rustfmt::skip]
    let texels: Vec<u32> = vec![
        0xFFFFFFFF, 0x000000FF, 0xFFFFFFFF, 0x000000FF,
        0x000000FF, 0xFFFFFFFF, 0x000000FF, 0xFFFFFFFF,
        0xFFFFFFFF, 0x000000FF, 0xFFFFFFFF, 0x000000FF,
        0x000000FF, 0xFFFFFFFF, 0x000000FF, 0xFFFFFFFF,
    ];

    // Create 4 by 4 texture and sampler.
    let texture = ctx.create_texture(texels.as_slice(), 4, 4);
    let sampler = ctx.create_sampler(Filter::Nearest, Filter::Nearest);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sprite batch
    ///////////////////////////////////////////////////////////////////////////

    let mut batch = SpriteBatch::new(&texture);

    let (sw, sh) = (64.0, 64.0);

    for i in 0..16 {
        for j in 0..16 {
            let x = i as f32 * sw * 2.0;
            let y = j as f32 * sh * 2.0;

            batch.add(
                Rect {
                    x1: 0.0,
                    y1: 0.0,
                    x2: texture.w as f32,
                    y2: texture.h as f32,
                },
                Rect {
                    x1: x,
                    y1: y,
                    x2: x + sw,
                    y2: y + sh,
                },
                1.0,
                1.0,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            );
        }
    }
    let vertex_buf = batch.finish(&ctx);

    ///////////////////////////////////////////////////////////////////////////
    // Setup transform & ortho uniforms
    ///////////////////////////////////////////////////////////////////////////

    #[derive(Copy, Clone)]
    struct Uniforms {
        pub ortho: Matrix4<f32>,
        pub transform: Matrix4<f32>,
    }

    let uniform_buf = ctx.create_uniform_buffer(Uniforms {
        ortho: kit.ortho.into(),
        transform: kit.transform,
    });

    ///////////////////////////////////////////////////////////////////////////
    // Setup uniform layout
    ///////////////////////////////////////////////////////////////////////////

    let mut uniforms_binding = UniformsBinding::from(&uniforms_layout);

    uniforms_binding[0] = Uniform::Buffer(&uniform_buf);
    uniforms_binding[1] = Uniform::Texture(&texture);
    uniforms_binding[2] = Uniform::Sampler(&sampler);

    let uniforms = ctx.create_uniforms(&uniforms_binding);

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;

    let mut running = true;

    while running {
        x += 1.0;
        y += 1.0;

        ///////////////////////////////////////////////////////////////////////////
        // Process events
        ///////////////////////////////////////////////////////////////////////////

        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        if let VirtualKeyCode::Escape = code {
                            running = false;
                        }
                    }
                    WindowEvent::CloseRequested => {
                        running = false;
                    }
                    _ => {}
                }
            }
        });

        ///////////////////////////////////////////////////////////////////////////
        // Update uniforms
        ///////////////////////////////////////////////////////////////////////////

        ctx.update_uniform_buffer(
            &uniform_buf,
            Uniforms {
                transform: Matrix4::from_translation(Vector3::new(x, y, 0.0)),
                ortho: kit.ortho.into(),
            },
        );

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        ctx.frame(|frame| {
            let mut pass = frame.begin_pass();

            pass.apply_pipeline(&pipeline);
            pass.apply_uniforms(&uniforms);
            pass.set_vertex_buffer(&vertex_buf);

            batch.draw(&mut pass);
        });
    }
}
