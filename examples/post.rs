#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate env_logger;
extern crate rgx;

use rgx::core::*;
use rgx::kit::*;

use image::ImageDecoder;

use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

use std::time::Instant;

fn main() {
    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup rgx context
    ///////////////////////////////////////////////////////////////////////////

    let mut kit = Kit::new(&window);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = kit.sampler(Filter::Nearest, Filter::Nearest);

    let sprite = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (w, h) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();

        kit.texture(pixels.as_slice(), w as u32, h as u32)
    };

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let w = 50.0;

    let mut anim = {
        let delay = 120.0; // Frame delay

        Animation::new(
            &[
                Rect::new(w * 0.0, 0.0, w * 1.0, sprite.h as f32),
                Rect::new(w * 1.0, 0.0, w * 2.0, sprite.h as f32),
                Rect::new(w * 2.0, 0.0, w * 3.0, sprite.h as f32),
                Rect::new(w * 3.0, 0.0, w * 4.0, sprite.h as f32),
                Rect::new(w * 4.0, 0.0, w * 5.0, sprite.h as f32),
                Rect::new(w * 5.0, 0.0, w * 6.0, sprite.h as f32),
            ],
            delay,
        )
    };

    let mut delta: f64;
    let mut last_frame = Instant::now();

    let mut running = true;

    let mut win = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let mut canvas = kit.canvas(win);

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
                        win = size.to_physical(window.get_hidpi_factor());
                        kit.resize(win);
                        canvas = kit.canvas(win);
                    }
                    _ => {}
                }
            }
        });

        {
            let now = Instant::now();
            delta = now.duration_since(last_frame).as_millis() as f64;
            last_frame = now;
        }

        ///////////////////////////////////////////////////////////////////////////
        // Update state
        ///////////////////////////////////////////////////////////////////////////

        anim.step(delta as f64);

        ///////////////////////////////////////////////////////////////////////////
        // Prepare frame
        ///////////////////////////////////////////////////////////////////////////

        let mut sb = SpriteBatch::new(&sprite, &sampler);
        let (sw, sh) = (w * 10.0, sprite.h as f32 * 10.0);

        let x = win.width as f32 / 2.0;
        let y = win.height as f32 / 2.0;

        sb.add(
            anim.val(),
            Rect::new(x, y, x + sw, y + sh),
            Rgba::TRANSPARENT,
            Repeat::default(),
        );
        sb.finish(&kit);

        ///////////////////////////////////////////////////////////////////////////
        // Draw to off-screen canvas
        ///////////////////////////////////////////////////////////////////////////

        kit.offscreen(&canvas, |c| {
            c.draw(&sb);
        });

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        kit.frame(|pass| {
            pass.draw(&canvas);
        });
    }
}
