#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate rgx;

use rgx::*;

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
    // Setup shader bindings layout
    ///////////////////////////////////////////////////////////////////////////

    let bindings_layout = ctx.create_bindings_layout(&[
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
    // Setup vertex & index buffers
    ///////////////////////////////////////////////////////////////////////////

    #[derive(Copy, Clone)]
    #[rustfmt::skip]
    struct Vertex(
        f32, f32,               // X Y
        f32, f32, f32, f32,     // R G B A
        f32, f32                // U V
    );

    #[rustfmt::skip]
    let vertex_buf = ctx.create_buffer(vec![
        //     X      Y        R    G    B    A      U    V
        Vertex(0.0,   0.0,     0.0, 0.0, 1.0, 1.0,   0.0, 0.0),
        Vertex(100.0, 0.0,     0.0, 1.0, 0.0, 1.0,   1.0, 0.0),
        Vertex(0.0,   100.0,   1.0, 0.0, 0.0, 1.0,   0.0, 1.0),
        Vertex(100.0, 100.0,   1.0, 1.0, 0.0, 1.0,   1.0, 1.0),
    ]);

    let index_buf = ctx.create_index(&[0, 1, 2, 2, 3, 1]);

    let vertex_layout = VertexLayout::from(&[
        VertexFormat::Float2,
        VertexFormat::Float4,
        VertexFormat::Float2,
    ]);

    ///////////////////////////////////////////////////////////////////////////
    // Setup render pipeline
    ///////////////////////////////////////////////////////////////////////////

    let pipeline = ctx.create_pipeline(&bindings_layout, &vertex_layout, &vs, &fs);

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
    let texture = ctx.create_texture(texels, 4, 4);
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

    let uniform_buf = ctx.create_uniform_buffer(Uniforms {
        ortho,
        transform: Matrix4::identity(),
    });

    ///////////////////////////////////////////////////////////////////////////
    // Setup uniform layout
    ///////////////////////////////////////////////////////////////////////////

    let mut slots = BindingSlots::from(&bindings_layout);

    slots[0] = Binding::UniformBuffer(&uniform_buf);
    slots[1] = Binding::Texture(&texture);
    slots[2] = Binding::Sampler(&sampler);

    let bindings = ctx.create_binding(&slots);

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;

    let mut running = true;

    while running {
        x += 1.0;
        y += 1.0;

        events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match code {
                    VirtualKeyCode::Escape => {
                        running = false;
                    }
                    _ => {}
                },
                WindowEvent::CloseRequested => {
                    running = false;
                }
                _ => {}
            },
            _ => {}
        });

        ctx.update_uniform_buffer(
            &uniform_buf,
            Uniforms {
                transform: Matrix4::from_translation(Vector3::new(x, y, 0.0)),
                ortho,
            },
        );

        ctx.frame(|frame| {
            let mut pass = frame.begin_pass();

            pass.wgpu.set_pipeline(&pipeline.wgpu);
            pass.wgpu.set_bind_group(0, &bindings.wgpu);
            pass.wgpu.set_index_buffer(&index_buf.wgpu, 0);
            pass.wgpu.set_vertex_buffers(&[(&vertex_buf.wgpu, 0)]);
            pass.wgpu.draw_indexed(0..6, 0, 0..1);
        });
    }
}
