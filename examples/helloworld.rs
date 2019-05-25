#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate env_logger;
extern crate rgx;

use rgx::core::*;
use rgx::kit;

use cgmath::prelude::*;
use cgmath::Matrix4;
use wgpu::winit::*;

fn main() {
    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();
    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    // Setup renderer
    let mut renderer = Renderer::new(&window);

    // Setup render pipeline
    let pipeline: kit::sprite2d::Pipeline =
        renderer.pipeline(size.width as u32, size.height as u32);

    // Setup texture & sampler
    #[rustfmt::skip]
    let texels: [u32; 16] = [
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
    ];
    let buf: [u8; 64] = unsafe { std::mem::transmute(texels) };

    // Create 4 by 4 texture and sampler.
    let texture = renderer.texture(&buf, 4, 4);
    let sampler = renderer.sampler(Filter::Nearest, Filter::Nearest);

    // Setup sprite
    let binding = pipeline.binding(&renderer, &texture, &sampler);
    let buffer = pipeline.sprite(
        &renderer,
        &texture,
        texture.rect(),
        Rect::new(0., 0., size.width as f32, size.height as f32),
        Rgba::new(0.5, 0.6, 0.8, 1.0),
        kit::Repeat::new(24. * (size.width / size.height) as f32, 24.),
    );

    let mut running = true;

    // Prepare resources
    renderer.prepare(&[&texture]);

    while running {
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
                    _ => {}
                }
            }
        });

        // Create frame
        let mut frame = renderer.frame();

        // Prepare pipeline
        frame.prepare(&pipeline, Matrix4::identity());

        // Draw frame
        let pass = &mut frame.pass(Rgba::TRANSPARENT);

        pass.apply_pipeline(&pipeline);
        pass.draw(&buffer, &binding);
    }
}
