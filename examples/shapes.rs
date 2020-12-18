#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Batch, Fill, Shape};

use rgx::math::*;

use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() -> Result<(), std::io::Error> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = futures::executor::block_on(Renderer::new(&window))?;
    let mut win = window.inner_size();

    let pip: kit::shape2d::Pipeline = r.pipeline(Blending::default());

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let (sw, sh) = (32., 32.);

    // Cursor position.
    let (mut mx, mut my) = (0., 0.);

    let mut textures = r.swap_chain(win.width as u32, win.height as u32, PresentMode::default());

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            window.request_redraw();
            *control_flow = ControlFlow::Wait;
        }
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
                mx = position.x as f32;
                my = position.y as f32;
                window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                win = size;

                let (w, h) = (win.width as u32, win.height as u32);
                textures = r.swap_chain(w, h, PresentMode::default());
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {
            eprintln!("REDRAW");
            let rows = (win.height as f32 / sh) as u32;
            let cols = (win.width as f32 / sw) as u32;

            ///////////////////////////////////////////////////////////////////////////
            // Prepare shape view
            ///////////////////////////////////////////////////////////////////////////

            let mut batch = Batch::new();
            let (dx, dy) = (
                (mx / win.width as f32) as f32,
                (my / win.height as f32) as f32,
            );

            for i in 0..rows {
                let y = i as f32 * sh;

                for j in 0..cols {
                    let x = j as f32 * sw - sw / 2.0;

                    let c1 = i as f32 / rows as f32 + dy;
                    let c2 = j as f32 / cols as f32 - dx;

                    if j % 2 == 0 && i % 2 == 0 {
                        batch.add(
                            Shape::circle(Point2::new(x + sw / 2., y + sw / 2.), sw * 2., 32)
                                .stroke(1.0, Rgba::new(0.5, c2, c1, 0.75)),
                        );
                    }

                    if j * i % 2 != 0 {
                        batch.add(
                            Shape::rect([x, y], [x + sw, y + sh])
                                .stroke(3.0, Rgba::new(c1, c2, 0.5, 1.0))
                                .fill(Fill::Solid(Rgba::new(1.0, dx, dy, 0.1))),
                        );
                    } else {
                        batch.add(Shape::line([x, y], [x + sw, y + sh]).stroke(
                            1.0,
                            Rgba::new(
                                i as f32 / rows as f32 + dy,
                                j as f32 / cols as f32 - dx,
                                0.5,
                                0.75,
                            ),
                        ));
                    };
                }
            }

            let buffer = batch.finish(&r);

            ///////////////////////////////////////////////////////////////////////////
            // Create frame
            ///////////////////////////////////////////////////////////////////////////

            let mut frame = r.frame();
            let out = textures.next().unwrap();

            r.update_pipeline(
                &pip,
                kit::ortho(out.width, out.height, Default::default()),
                &mut frame,
            );

            ///////////////////////////////////////////////////////////////////////////
            // Draw frame
            ///////////////////////////////////////////////////////////////////////////

            {
                let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

                pass.set_pipeline(&pip);
                pass.draw_buffer(&buffer);
            }
            dbg!("4");
            r.present(frame);
            dbg!("5");
        }
        _ => {}
    });
}
