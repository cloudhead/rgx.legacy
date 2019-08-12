#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::sprite2d;

use cgmath::{Matrix4, Vector3};

use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

fn main() {
    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut renderer = Renderer::new(&window);

    ///////////////////////////////////////////////////////////////////////////
    // Setup render pipeline
    ///////////////////////////////////////////////////////////////////////////

    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let mut pipeline: kit::sprite2d::Pipeline =
        renderer.pipeline(size.width as u32, size.height as u32, Blending::default());

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
    let buf: [u8; 64] = unsafe { std::mem::transmute(texels) };

    // Create 4 by 4 texture and sampler.
    let texture = renderer.texture(4, 4);
    let sampler = renderer.sampler(Filter::Nearest, Filter::Nearest);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sprites
    ///////////////////////////////////////////////////////////////////////////

    let binding = pipeline.binding(&renderer, &texture, &sampler);

    let bg_buffer = sprite2d::Batch::singleton(
        texture.w,
        texture.h,
        texture.rect(),
        Rect::new(0., 0., size.width as f32, size.height as f32),
        Rgba::TRANSPARENT,
        1.0,
        kit::Repeat::new(24. * (size.width / size.height) as f32, 24.),
    )
    .finish(&renderer);

    let fg_buffer = sprite2d::Batch::singleton(
        texture.w,
        texture.h,
        texture.rect(),
        Rect::origin(160.0, 160.0),
        Rgba::new(1.0, 1.0, 0.0, 0.5),
        1.0,
        kit::Repeat::default(),
    )
    .finish(&renderer);

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;

    let mut running = true;
    let mut transform;
    let mut textures = renderer.swap_chain(
        size.width as u32,
        size.height as u32,
        PresentMode::default(),
    );

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    renderer.prepare(&[Op::Fill(&texture, &buf)]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    while running {
        x += 1.0;
        y += 1.0;

        transform = Matrix4::from_translation(Vector3::new(x, y, 0.0));

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
                        let physical = size.to_physical(window.get_hidpi_factor());
                        let (w, h) = (physical.width as u32, physical.height as u32);

                        pipeline.resize(w, h);
                        textures = renderer.swap_chain(w, h, PresentMode::default());
                    }
                    _ => {}
                }
            }
        });

        ///////////////////////////////////////////////////////////////////////////
        // Create output texture
        ///////////////////////////////////////////////////////////////////////////

        let out = textures.next();

        ///////////////////////////////////////////////////////////////////////////
        // Create frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = renderer.frame();

        ///////////////////////////////////////////////////////////////////////////
        // Update uniform
        ///////////////////////////////////////////////////////////////////////////

        renderer.update_pipeline(&pipeline, transform, &mut frame);

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        {
            let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

            pass.set_pipeline(&pipeline);
            pass.draw(&bg_buffer, &binding);
            pass.draw(&fg_buffer, &binding);
        }
        renderer.submit(frame);
    }
}
