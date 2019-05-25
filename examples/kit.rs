#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate env_logger;
extern crate rgx;

use rgx::core::*;
use rgx::kit;

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

    let mut pipeline: kit::sprite2d::Pipeline = renderer.pipeline(
        kit::sprite2d::SPRITE2D,
        size.width as u32,
        size.height as u32,
    );

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
    let texture = renderer.texture(&buf, 4, 4);
    let sampler = renderer.sampler(Filter::Nearest, Filter::Nearest);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sprites
    ///////////////////////////////////////////////////////////////////////////

    let binding = pipeline.binding(&renderer, &texture, &sampler);

    let buffer_bg = pipeline.sprite(
        &renderer,
        &texture,
        texture.rect(),
        Rect::new(0., 0., size.width as f32, size.height as f32),
        Rgba::TRANSPARENT,
        kit::Repeat::new(24. * (size.width / size.height) as f32, 24.),
    );

    let buffer_fg = pipeline.sprite(
        &renderer,
        &texture,
        texture.rect(),
        Rect::new(0.0, 0.0, 160.0, 160.0),
        Rgba::new(1.0, 1.0, 0.0, 0.5),
        kit::Repeat::default(),
    );

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;

    let mut running = true;
    let mut transform;

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    renderer.prepare(&[&texture]);

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
                        renderer.resize(w, h);
                    }
                    _ => {}
                }
            }
        });

        ///////////////////////////////////////////////////////////////////////////
        // Create frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = renderer.frame();

        ///////////////////////////////////////////////////////////////////////////
        // Prepare pipeline
        ///////////////////////////////////////////////////////////////////////////

        frame.prepare(&pipeline, transform);

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        let pass = &mut frame.pass(Rgba::TRANSPARENT);

        pass.apply_pipeline(&pipeline);
        pass.draw(&buffer_bg, &binding);
        pass.draw(&buffer_fg, &binding);
    }
}
