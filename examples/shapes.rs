#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Batch, Fill, Line, Shape, Stroke};

use rgx::math::*;

use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = Renderer::new(window.raw_window_handle());
    let mut win = window.inner_size().to_physical(window.hidpi_factor());

    let mut pip: kit::shape2d::Pipeline =
        r.pipeline(win.width as u32, win.height as u32, Blending::default());

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let (sw, sh) = (32., 32.);

    // Cursor position.
    let (mut mx, mut my) = (0., 0.);

    let mut textures = r.swap_chain(win.width as u32, win.height as u32, PresentMode::default());
    
    let mut redraw = true;

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
                redraw = true;
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                win = size.to_physical(window.hidpi_factor());

                let (w, h) = (win.width as u32, win.height as u32);

                pip.resize(w, h);
                textures = r.swap_chain(w, h, PresentMode::default());
            }
            WindowEvent::RedrawRequested => {
                redraw = true;
            }
            _ => {}
        },
        Event::EventsCleared => {
            if !redraw {
                // By only redrawing as necessary we reduce CPU usage
                return;
            }
            *control_flow = ControlFlow::Wait;
            redraw = false;

            let rows = (win.height as f32 / sh) as u32;
            let cols = (win.width as f32 / sw) as u32;

            ///////////////////////////////////////////////////////////////////////////
            // Prepare shape view
            ///////////////////////////////////////////////////////////////////////////

            let mut batch = Batch::new();
            let (dx, dy) = ((mx / win.width) as f32, (my / win.height) as f32);

            for i in 0..rows {
                let y = i as f32 * sh;

                for j in 0..cols {
                    let x = j as f32 * sw - sw / 2.0;

                    let c1 = i as f32 / rows as f32 + dy;
                    let c2 = j as f32 / cols as f32 - dx;

                    if j % 2 == 0 && i % 2 == 0 {
                        batch.add(Shape::Circle(
                            Point2::new(x + sw / 2., y + sw / 2.),
                            sw * 2.,
                            32,
                            Stroke::new(1.0, Rgba::new(0.5, c2, c1, 0.75)),
                            Fill::Empty(),
                        ));
                    }

                    if j * i % 2 != 0 {
                        batch.add(Shape::Rectangle(
                            Rect::new(x, y, x + sw, y + sh),
                            Stroke::new(3.0, Rgba::new(c1, c2, 0.5, 1.0)),
                            Fill::Solid(Rgba::new(1.0, dx, dy, 0.1)),
                        ));
                    } else {
                        batch.add(Shape::Line(
                            Line::new(x, y, x + sw, y + sh),
                            Stroke::new(
                                1.0,
                                Rgba::new(
                                    i as f32 / rows as f32 + dy,
                                    j as f32 / cols as f32 - dx,
                                    0.5,
                                    0.75,
                                ),
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

            ///////////////////////////////////////////////////////////////////////////
            // Draw frame
            ///////////////////////////////////////////////////////////////////////////

            let out = textures.next();
            {
                let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

                pass.set_pipeline(&pip);
                pass.draw_buffer(&buffer);
            }
            r.submit(frame);
        }
        _ => {}
    });
}
