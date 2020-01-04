#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit::shape2d::{Batch, Fill, Shape, Stroke};
use rgx::kit::{self, ZDepth};

use rgx::math::*;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = Renderer::new(&window)?;
    let mut win = window.inner_size().to_physical(window.hidpi_factor());

    let pip: kit::shape2d::Pipeline = r.pipeline(Blending::default());
    let mut chain = r.swap_chain(win.width as u32, win.height as u32, PresentMode::default());

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let rad = 64.;

    // Cursor position.
    let (mut mx, mut my) = (0., 0.);

    event_loop.run(move |event, _, control_flow| match event {
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
            WindowEvent::CursorMoved { position, .. } => {
                mx = position.x;
                my = position.y;
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                win = size.to_physical(window.hidpi_factor());

                let (w, h) = (win.width as u32, win.height as u32);
                chain = r.swap_chain(w, h, PresentMode::default());
            }
            _ => {}
        },
        Event::EventsCleared => {
            *control_flow = ControlFlow::Wait;

            let rows = (win.height as f32 / (rad * 2.)) as u32;
            let cols = (win.width as f32 / (rad * 2.)) as u32;

            ///////////////////////////////////////////////////////////////////////////
            // Prepare shape view
            ///////////////////////////////////////////////////////////////////////////

            let mut batch = Batch::new();
            let cursor = Vector2::new((mx / win.width) as f32, 1. - (my / win.height) as f32);

            for i in 0..rows {
                let y = i as f32 * rad * 2.;

                for j in 0..cols {
                    let x = j as f32 * rad * 2.;

                    // Color
                    let c1 = i as f32 / rows as f32;
                    let c2 = j as f32 / cols as f32;

                    let rpos = Vector2::new(i as f32 / rows as f32, j as f32 / cols as f32);
                    let delta = Vector2::distance(rpos, cursor);
                    let width = 1. + delta * (rad / 1.5);

                    batch.add(Shape::Circle(
                        Point2::new(x + rad, y + rad),
                        ZDepth::ZERO,
                        rad,
                        32,
                        Stroke::new(width, Rgba::new(0.5, c2, c1, 0.75)),
                        Fill::Empty(),
                    ));
                }
            }

            let buffer = batch.finish(&r);

            ///////////////////////////////////////////////////////////////////////////
            // Create frame
            ///////////////////////////////////////////////////////////////////////////

            let out = chain.next();
            let mut frame = r.frame();

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
                pass.draw_buffer(&buffer);
            }
            r.present(frame);
        }
        _ => {}
    });
}
