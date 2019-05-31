#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use std::collections::vec_deque::VecDeque;

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

use std::time::Instant;

fn main() {
    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = Renderer::new(&window);

    let mut win = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let mut pip: kit::sprite2d::Pipeline = r.pipeline(win.width as u32, win.height as u32);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);

    let sprite = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (w, h) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();

        r.texture(pixels.as_slice(), w as u32, h as u32)
    };

    let binding = pip.binding(&r, &sprite, &sampler); // Texture binding

    let sprite_w = 50.0;
    let sprite_h = sprite.h as f32;

    let mut anim = {
        let delay = 160.0; // Frame delay

        Animation::new(
            &[
                Rect::new(sprite_w * 0.0, 0.0, sprite_w * 1.0, sprite_h),
                Rect::new(sprite_w * 1.0, 0.0, sprite_w * 2.0, sprite_h),
                Rect::new(sprite_w * 2.0, 0.0, sprite_w * 3.0, sprite_h),
                Rect::new(sprite_w * 3.0, 0.0, sprite_w * 4.0, sprite_h),
                Rect::new(sprite_w * 4.0, 0.0, sprite_w * 5.0, sprite_h),
                Rect::new(sprite_w * 5.0, 0.0, sprite_w * 6.0, sprite_h),
            ],
            delay,
        )
    };

    let move_speed = 8.0;
    let mut x = 0.0;

    let frame_batch = 120;
    let mut delta: f64;
    let mut last_frame = Instant::now();
    let mut fts: VecDeque<f64> = VecDeque::with_capacity(frame_batch);
    let mut average_ft: f64;
    let mut frames_total = 0;

    let mut running = true;

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    r.prepare(&[&sprite]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let mut mx: f32 = 0.;
    let mut my: f32 = 0.;

    let mut scale = 1.0;
    let (mut sw, mut sh);
    let mut rows: u32;
    let mut cols: u32;

    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match key {
                        VirtualKeyCode::Escape => {
                            running = false;
                        }
                        VirtualKeyCode::Up => {
                            my += 24.;
                        }
                        VirtualKeyCode::Down => {
                            my -= 24.;
                        }
                        VirtualKeyCode::Left => {
                            mx -= 24.;
                        }
                        VirtualKeyCode::Right => {
                            mx += 24.;
                        }
                        _ => {}
                    },
                    WindowEvent::CloseRequested => {
                        running = false;
                    }
                    WindowEvent::Resized(size) => {
                        win = size.to_physical(window.get_hidpi_factor());

                        let (w, h) = (win.width as u32, win.height as u32);

                        pip.resize(w, h);
                        r.resize(w, h);
                    }
                    _ => {}
                }
            }
        });

        sw = sprite_w as f32 * scale;
        sh = sprite_h as f32 * scale;
        rows = (win.height as f32 / sh) as u32;
        cols = (win.width as f32 / (sw / 2.0)) as u32;

        {
            let now = Instant::now();
            delta = now.duration_since(last_frame).as_millis() as f64;
            last_frame = now;
        }

        ///////////////////////////////////////////////////////////////////////////
        // Update state
        ///////////////////////////////////////////////////////////////////////////

        anim.step(delta as f64);
        fts.push_front(delta as f64);
        fts.truncate(frame_batch);

        ///////////////////////////////////////////////////////////////////////////
        // Prepare sprite batch
        ///////////////////////////////////////////////////////////////////////////

        let win = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());

        let mut tv = TextureView::new(sprite.w, sprite.h);

        x += delta as f32 / move_speed;

        for i in 0..rows {
            let y = i as f32 * sh;

            for j in 0..cols {
                let pad = j as f32 * sw / 2.0;

                let rect = if i % 2 == 0 {
                    Rect::new(
                        win.width as f32 - x - pad,
                        y,
                        win.width as f32 - x - pad - sw,
                        y + sh,
                    )
                } else {
                    Rect::new(pad + x, y, pad + x + sw, y + sh)
                };

                tv.add(
                    anim.val(),
                    rect,
                    Rgba::new(i as f32 / rows as f32, j as f32 / cols as f32, 0.5, 0.75),
                    Repeat::default(),
                );
            }
        }
        tv.offset(mx, my);

        let buffer = tv.finish(&r);

        ///////////////////////////////////////////////////////////////////////////
        // Create frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = r.frame();

        ///////////////////////////////////////////////////////////////////////////
        // Prepare pipeline
        ///////////////////////////////////////////////////////////////////////////

        frame.prepare(&pip, Matrix4::identity());

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        let pass = &mut frame.pass(Rgba::TRANSPARENT);

        pass.apply_pipeline(&pip);
        pass.draw(&buffer, &binding);

        if frames_total >= frame_batch && frames_total % frame_batch == 0 {
            average_ft = fts.iter().sum::<f64>() / fts.len() as f64;

            println!("sprites/frame: {}", rows * cols);
            println!("time/frame:    {:.2}ms\n", average_ft);

            if average_ft as u32 <= 16 && scale > 0.1 {
                scale -= 0.1;
                x = 0.;
            } else {
                break;
            }
        }
        frames_total += 1;
    }
}
