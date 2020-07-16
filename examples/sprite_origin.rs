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
 use rgx::math::Vector2;

 fn main() -> Result<(), std::io::Error> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut renderer = Renderer::new(&window)?;
    let mut window = window.inner_size();
    let sprite_pipeline: kit::sprite2d::Pipeline = renderer.pipeline(Blending::default());

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = renderer.sampler(Filter::Nearest, Filter::Nearest);

    let (sprite, texels) = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (width, height) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();
        let pixels = Rgba8::align(&pixels);

        (
            renderer.texture(width as u32, height as u32),
            pixels.to_owned(),
        )
    };

    let binding = sprite_pipeline.binding(&renderer, &sprite, &sampler); // Texture binding

    let sprite_width = 50.0;
    let sprite_height = sprite.h as f32;

    let src = Rect::new(sprite_width * 0.0, 0.0, sprite_width * 1.0, sprite_height);

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    renderer.submit(&[Op::Fill(&sprite, texels.as_slice())]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////
    let mut scale = 1.0;
    let mut textures = renderer.swap_chain(
        window.width as u32,
        window.height as u32,
        PresentMode::default(),
    );

    let mut last_frame = std::time::Instant::now();
    let mut delta = 0.0;
    let mut seconds = 0.0;
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
                    _ => {}
                },
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    window = size;

                    let (window_width, window_height) = (window.width as u32, window.height as u32);
                    textures = renderer.swap_chain(window_width, window_height, PresentMode::default());
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                *control_flow = ControlFlow::Poll;

                ///////////////////////////////////////////////////////////////////////////
                // Prepare sprite batch
                ///////////////////////////////////////////////////////////////////////////

                let mut batch = sprite2d::Batch::new(sprite.w, sprite.h);
                let scale = f32::abs(f32::sin(seconds * 3.14 / 4.0)) * 10.0;
                batch.add(
                    src,
                    Vector2 { x: window.width as f32 / 2.0, y: window.height as f32 / 2.0 },
                    seconds * 360.0 / 2.0,
                    Vector2::new(scale, scale),
                    Vector2::new(0.5, 0.5),
                    ZDepth::default(),
                    Rgba::new(1.0, 1.0, 1.0, 0.0),
                    1.0,
                    Repeat::default(),
                );

                let buffer = batch.finish(&renderer);

                ///////////////////////////////////////////////////////////////////////////
                // Create frame & output
                ///////////////////////////////////////////////////////////////////////////

                let mut frame = renderer.frame();
                let out = textures.next();

                ///////////////////////////////////////////////////////////////////////////
                // Draw frame
                ///////////////////////////////////////////////////////////////////////////

                renderer.update_pipeline(
                    &sprite_pipeline,
                    kit::ortho(out.width, out.height, Default::default()),
                    &mut frame,
                );
                {
                    let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

                    pass.set_pipeline(&sprite_pipeline);
                    pass.draw(&buffer, &binding);
                }

                renderer.present(frame);

                delta = (std::time::Instant::now() - last_frame).as_secs_f32();
                seconds += delta;
                last_frame = std::time::Instant::now();
            }
            _ => (),
        }
    });
}
