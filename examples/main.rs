#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate rgx;

use rgx::core::*;

use cgmath::prelude::*;
use cgmath::{Matrix4, Ortho, Vector3};

use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup rgx context
    ///////////////////////////////////////////////////////////////////////////

    let mut ctx = Context::new(&window);

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
    // Setup shader pipeline layout
    ///////////////////////////////////////////////////////////////////////////

    let pipeline_layout = ctx.create_pipeline_layout(&[Set(&[
        Binding {
            binding: BindingType::UniformBuffer,
            stage: ShaderStage::Vertex,
        },
        Binding {
            binding: BindingType::SampledTexture,
            stage: ShaderStage::Fragment,
        },
        Binding {
            binding: BindingType::Sampler,
            stage: ShaderStage::Fragment,
        },
    ])]);

    ///////////////////////////////////////////////////////////////////////////
    // Setup vertex & index buffers
    ///////////////////////////////////////////////////////////////////////////

    #[derive(Copy, Clone)]
    #[rustfmt::skip]
    struct Vertex(
        f32, f32,               // X Y
        f32, f32,               // U V
        f32, f32, f32, f32,     // R G B A
    );

    #[rustfmt::skip]
    let vertex_buf = ctx.create_buffer(vec![
        //     X      Y       U    V      R    G    B    A
        Vertex(0.0,   0.0,    0.0, 0.0,   0.0, 0.0, 1.0, 1.0),
        Vertex(100.0, 0.0,    1.0, 0.0,   0.0, 1.0, 0.0, 1.0),
        Vertex(0.0,   100.0,  0.0, 1.0,   1.0, 0.0, 0.0, 1.0),
        Vertex(100.0, 100.0,  1.0, 1.0,   1.0, 1.0, 0.0, 1.0),
    ].as_slice());

    let index_buf = ctx.create_index(&[0, 1, 2, 2, 3, 1]);

    let vertex_layout = VertexLayout::from(&[
        VertexFormat::Float2,
        VertexFormat::Float2,
        VertexFormat::Float4,
    ]);

    ///////////////////////////////////////////////////////////////////////////
    // Setup render pipeline
    ///////////////////////////////////////////////////////////////////////////

    let pipeline = ctx.create_pipeline(pipeline_layout, vertex_layout, &vs, &fs);

    ///////////////////////////////////////////////////////////////////////////
    // Setup texture & sampler
    ///////////////////////////////////////////////////////////////////////////

    #[rustfmt::skip]
    let texels: [u32; 16] = [
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
    ];
    let buffer: [u8; 64] = unsafe { std::mem::transmute(texels) };

    // Create 4 by 4 texture and sampler.
    let texture = ctx.create_texture(&buffer, 4, 4);
    let sampler = ctx.create_sampler(Filter::Nearest, Filter::Nearest);

    ///////////////////////////////////////////////////////////////////////////
    // Setup transform & ortho uniforms
    ///////////////////////////////////////////////////////////////////////////

    #[derive(Copy, Clone)]
    struct Uniforms {
        pub ortho: Matrix4<f32>,
        pub transform: Matrix4<f32>,
    }

    let win_size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let ortho: Matrix4<f32> = Ortho::<f32> {
        left: 0.0,
        right: win_size.width as f32,
        bottom: 0.0,
        top: win_size.height as f32,
        near: -1.0,
        far: 1.0,
    }
    .into();

    let uniform_buf = std::rc::Rc::new(ctx.create_uniform_buffer(Uniforms {
        ortho,
        transform: Matrix4::identity(),
    }));

    ///////////////////////////////////////////////////////////////////////////
    // Setup uniform layout
    ///////////////////////////////////////////////////////////////////////////

    let uniforms = ctx.create_binding(
        &pipeline.layout.sets[0],
        &[
            Uniform::Buffer(&uniform_buf),
            Uniform::Texture(&texture),
            Uniform::Sampler(&sampler),
        ],
    );

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
                    WindowEvent::Resized(size) => {
                        ctx.resize(size.to_physical(window.get_hidpi_factor()));
                    }
                    _ => {}
                }
            }
        });

        ///////////////////////////////////////////////////////////////////////////
        // Update uniforms
        ///////////////////////////////////////////////////////////////////////////

        ctx.update_uniform_buffer(
            uniform_buf.clone(),
            Uniforms {
                transform: Matrix4::from_translation(Vector3::new(x, y, 0.0)),
                ortho,
            },
        );

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = ctx.frame();
        {
            let mut pass = frame.begin_pass();

            pass.apply_pipeline(&pipeline);
            pass.apply_uniforms(&uniforms);
            pass.set_index_buffer(&index_buf);
            pass.set_vertex_buffer(&vertex_buf);
            pass.draw_indexed(0..6, 0..1);
        }
        frame.commit();
    }
}
