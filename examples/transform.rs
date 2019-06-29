#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::sprite2d::TextureView;

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
        renderer.pipeline(size.width as u32, size.height as u32);

    ///////////////////////////////////////////////////////////////////////////
    // Setup texture & sampler
    ///////////////////////////////////////////////////////////////////////////

    #[rustfmt::skip]
    let texels: [u32; 4] = [
        0xFFFFFFFF, 0x00000000,
        0x00000000, 0xFFFFFFFF,
    ];
    let buf: [u8; 16] = unsafe { std::mem::transmute(texels) };

    // Create 4 by 4 texture and sampler.
    let texture = renderer.texture(&buf, 2, 2);
    let sampler = renderer.sampler(Filter::Nearest, Filter::Nearest);

    ///////////////////////////////////////////////////////////////////////////
    // Setup sprites
    ///////////////////////////////////////////////////////////////////////////

    let binding = pipeline.binding(&renderer, &texture, &sampler);

    let view_bg = TextureView::singleton(
        texture.w,
        texture.h,
        texture.rect(),
        Rect::new(0., 0., size.width as f32, size.height as f32),
        Rgba::new(0.2, 0.2, 0.2, 1.0),
        1.0,
        kit::Repeat::new(8. * (size.width / size.height) as f32, 8.),
    );
    let buffer_bg = view_bg.finish(&renderer);

    let view_fg = TextureView::singleton(
        texture.w,
        texture.h,
        texture.rect(),
        Rect::origin(160.0, 160.0),
        Rgba::new(0.9, 0.0, 0.0, 0.5),
        1.0,
        kit::Repeat::default(),
    );
    let buffer_fg = view_fg.finish(&renderer);

    let mut running = true;
    let base: f32 = 160.;
    let mut offset: f32 = 0.;

    ///////////////////////////////////////////////////////////////////////////
    // Prepare resources
    ///////////////////////////////////////////////////////////////////////////

    renderer.prepare(&[&texture]);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    while running {
        offset += 1.;

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
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        pipeline.frame(&mut renderer, Rgba::TRANSPARENT, |f| {
            f.draw(&buffer_bg, &binding);

            f.translate(base + offset, base, |f| {
                f.translate(base + offset, base, |f| {
                    f.draw(&buffer_fg, &binding);
                });
                f.draw(&buffer_fg, &binding);
            });
            if offset > 160.0 {
                f.transform(
                    Matrix4::from_translation(Vector3::new(base + offset, base * 2., 0.))
                        * Matrix4::from_scale(0.3),
                    |f| {
                        f.draw(&buffer_fg, &binding);
                    },
                );
            }
        });
    }
}
