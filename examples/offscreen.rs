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

    let (sw, sh) = (size.width as u32, size.height as u32);
    let mut offscreen: kit::sprite2d::Pipeline = r.pipeline(kit::sprite2d::SPRITE2D, sw, sh);
    let mut onscreen: kit::PipelinePost = r.pipeline(kit::FRAMEBUFFER, sw, sh);

    let framebuffer = onscreen.framebuffer(&r);

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

    let offscreen_binding = offscreen.binding(&r, &texture, &sampler); // Texture binding
    let onscreen_binding = onscreen.binding(&r, &framebuffer, &sampler);

    let w = 50.0;
    let rect = Rect::new(w * 1.0, 0.0, w * 2.0, texture.h as f32);
    let buffer = offscreen.sprite(
        &r,
        &texture,
        rect,
        Rect::new(0., 0., sw as f32, sh as f32),
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

                        offscreen.resize(w, h);
                        onscreen.resize(w, h);
                        r.resize(w, h);
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

        frame.prepare(&offscreen, Matrix4::identity());
        frame.prepare(&onscreen, Rgba::new(0.2, 0.2, 0.0, 1.0));

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        {
            let pass = &mut frame.offscreen_pass(Rgba::TRANSPARENT, &framebuffer.target);
            pass.apply_pipeline(&offscreen);
            pass.draw(&buffer, &offscreen_binding);
        }

        {
            let pass = &mut frame.pass(Rgba::TRANSPARENT);
            pass.apply_pipeline(&onscreen);
            pass.draw(&framebuffer, &onscreen_binding);
        }
    }
}
