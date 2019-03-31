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

    let mut kit = Kit::new(&window);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & texture & sampler
    ///////////////////////////////////////////////////////////////////////////

    let sampler = kit.sampler(Filter::Nearest, Filter::Nearest);

    #[rustfmt::skip]
    let texels_1: Vec<u32> = vec![
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
    ];
    let texture_1 = kit.texture(texels_1.as_slice(), 4, 4);

    #[rustfmt::skip]
    let texels_2 = vec![
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
    ];
    let texture_2 = kit.texture(texels_2.as_slice(), 4, 4);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sprite batches
    ///////////////////////////////////////////////////////////////////////////

    let mut batch_1 = SpriteBatch::new(&texture_1, &sampler);
    let (sw, sh) = (128.0, 128.0);

    for i in 0..16 {
        for j in 0..16 {
            let x = i as f32 * sw;
            let y = j as f32 * sh;

            batch_1.add(
                texture_1.rect(),
                Rect::new(x, y, x + sw, y + sh),
                Rgba::new(64, 64, 128, 255),
                Repeat::default(),
            );
        }
    }
    batch_1.finish(&kit);

    ///////////////////////////////////////////////////////////////////////////

    let mut batch_2 = SpriteBatch::new(&texture_2, &sampler);
    let (sw, sh) = (64.0, 64.0);

    for i in 0..16 {
        for j in 0..16 {
            let x = i as f32 * sw * 2.0;
            let y = j as f32 * sh * 2.0;

            batch_2.add(
                texture_1.rect(),
                Rect::new(x, y, x + sw, y + sh),
                Rgba::new(128, 64, 128, 255),
                Repeat::default(),
            );
        }
    }
    batch_2.finish(&kit);

    ///////////////////////////////////////////////////////////////////////////
    // Setup uniform layout
    ///////////////////////////////////////////////////////////////////////////

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
        // Update transform
        ///////////////////////////////////////////////////////////////////////////

        kit.transform = Matrix4::from_translation(Vector3::new(x, y, 0.0));

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = kit.frame();
        {
            let mut pass = frame.begin_pass();
            batch_1.draw(&mut pass);
            batch_2.draw(&mut pass);
        }
        frame.commit();
    }
}
