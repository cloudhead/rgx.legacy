#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

extern crate env_logger;
extern crate rgx;

use rgx::core::*;
use rgx::kit;
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

    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let mut pip_2d: kit::Pipeline2d =
        r.pipeline(kit::SPRITE2D, size.width as u32, size.height as u32);
    let mut pip_post: kit::Pipeline2d =
        r.pipeline(kit::FRAMEBUFFER, size.width as u32, size.height as u32);

    let mut framebuffer = r.framebuffer(w, h);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sampler & load texture
    ///////////////////////////////////////////////////////////////////////////

    let sampler = r.sampler(Filter::Nearest, Filter::Nearest);

    let texture = {
        let bytes = include_bytes!("data/sprite.tga");
        let tga = std::io::Cursor::new(bytes.as_ref());
        let decoder = image::tga::TGADecoder::new(tga).unwrap();
        let (w, h) = decoder.dimensions();
        let pixels = decoder.read_image().unwrap();

        r.texture(pixels.as_slice(), w as u32, h as u32)
    };

    let binding = pip_2d.binding(&r, &texture, &sampler); // Texture binding

    let w = 50.0;
    let rect = Rect::new(w * 1.0, 0.0, w * 2.0, texture.h as f32);
    let buffer = pip_2d.sprite(
        &r,
        &texture,
        rect,
        rect,
        Rgba::TRANSPARENT,
        Repeat::default(),
    );

    let mut running = true;

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    r.prepare(&[&texture]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

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
                        let physical = size.to_physical(window.get_hidpi_factor());
                        let (w, h) = (physical.width as u32, physical.height as u32);

                        pip_2d.resize(w, h);
                        pip_post.resize(w, h);
                        r.resize(w, h);
                        framebuffer = r.framebuffer(w, h); // TODO: Call 'resize'
                    }
                    _ => {}
                }
            }
        });

        ///////////////////////////////////////////////////////////////////////////
        // Create frame
        ///////////////////////////////////////////////////////////////////////////

        let mut frame = r.frame();

        ///////////////////////////////////////////////////////////////////////////
        // Prepare pipeline
        ///////////////////////////////////////////////////////////////////////////

        frame.prepare(&pip_2d, Matrix4::identity());
        frame.prepare(&pip_post, Matrix4::identity());

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        let offscreen = &mut frame.offscreen_pass(Rgba::TRANSPARENT, &framebuffer);
        offscreen.apply_pipeline(&pip_2d);
        offscreen.draw(&buffer, &binding);

        let onscreen = &mut frame.pass(Rgba::TRANSPARENT);
        onscreen.apply_pipeline(&pip_post);

        onscreen.apply_pipeline(&pip_post);
        onscreen.draw(&framebuffer, &binding);
    }
}
