#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use std::collections::vec_deque::VecDeque;

use rgx::core::*;
use rgx::kit;
use rgx::kit::sprite2d;
use rgx::kit::*;

use image::ImageDecoder;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use std::time::{Duration, Instant};

fn main() -> Result<(), std::io::Error> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = Renderer::new(&window)?;
    let mut win = window.inner_size();
    let pip: kit::sprite2d::Pipeline = r.pipeline(Blending::default());

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);

    let (sprite, texels) = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (w, h) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();
        let pixels = Rgba8::align(&pixels);

        (r.texture(w as u32, h as u32), pixels.to_owned())
    };

    let binding = pip.binding(&r, &sprite, &sampler); // Texture binding

    let sprite_w = 50.0;
    let sprite_h = sprite.h as f32;

    let mut anim = {
        let delay = Duration::from_millis(160); // Frame delay

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
    let mut last_frame = Instant::now();
    let mut fts: VecDeque<f64> = VecDeque::with_capacity(frame_batch);
    let mut frames_total = 0;

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    r.submit(&[Op::Fill(&sprite, texels.as_slice())]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let mut mx: f32 = 0.;
    let mut my: f32 = 0.;
    let mut scale = 1.0;
    let mut textures = r.swap_chain(win.width as u32, win.height as u32, PresentMode::default());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
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
                        *control_flow = ControlFlow::Exit;
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
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    win = size;

                    let (w, h) = (win.width as u32, win.height as u32);
                    textures = r.swap_chain(w, h, PresentMode::default());
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                *control_flow = ControlFlow::Poll;

                let sw = sprite_w as f32 * scale;
                let sh = sprite_h as f32 * scale;
                let rows = (win.height as f32 / sh) as u32;
                let cols = (win.width as f32 / (sw / 2.0)) as u32;

                let delta: Duration;
                {
                    let now = Instant::now();
                    delta = now.duration_since(last_frame);
                    last_frame = now;
                }

                ///////////////////////////////////////////////////////////////////////////
                // Update state
                ///////////////////////////////////////////////////////////////////////////

                anim.step(delta);
                fts.push_front(delta.as_millis() as f64);
                fts.truncate(frame_batch);

                ///////////////////////////////////////////////////////////////////////////
                // Prepare sprite batch
                ///////////////////////////////////////////////////////////////////////////

                let window_size = window.inner_size();

                let mut batch = sprite2d::Batch::new(sprite.w, sprite.h);

                x += delta.as_millis() as f32 / move_speed;

                for i in 0..rows {
                    let y = i as f32 * sh;

                    for j in 0..cols {
                        let src = anim.val();
                        let height_scaled = src.height() * scale;

                        let pad = j as f32 * sw / 2.0;

                        let pos = if i % 2 == 0 {
                            ultraviolet::Vec2::new(window_size.width as f32 - x - pad, y + height_scaled)
                        } else {
                            ultraviolet::Vec2::new(pad + x, y + height_scaled)
                        };

                        batch.add(
                            src,
                            pos,
                            180.0,
                            ultraviolet::Vec2::new(scale, scale),
                            ultraviolet::Vec2::new(0.5, 0.5),
                            ZDepth::default(),
                            Rgba::new(i as f32 / rows as f32, j as f32 / cols as f32, 0.5, 0.5),
                            1.0,
                            Repeat::default(),
                        );
                    }
                }
                batch.offset(mx, my);

                let buffer = batch.finish(&r);

                ///////////////////////////////////////////////////////////////////////////
                // Create frame & output
                ///////////////////////////////////////////////////////////////////////////

                let mut frame = r.frame();
                let out = textures.next();

                ///////////////////////////////////////////////////////////////////////////
                // Draw frame
                ///////////////////////////////////////////////////////////////////////////

                r.update_pipeline(
                    &pip,
                    kit::ortho(out.width, out.height, Default::default()),
                    &mut frame,
                );
                {
                    let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

                    pass.set_pipeline(&pip);
                    pass.draw(&buffer, &binding);
                }

                r.present(frame);

                if frames_total >= frame_batch && frames_total % frame_batch == 0 {
                    let average_ft = fts.iter().sum::<f64>() / fts.len() as f64;

                    println!("sprites/frame: {}", rows * cols);
                    println!("time/frame:    {:.2}ms\n", average_ft);

                    if average_ft as u32 <= 16 {
                        scale -= 0.1;
                        x = 0.;
                    } else {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                frames_total += 1;
            }
            _ => (),
        }
    });
}
