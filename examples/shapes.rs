#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Fill, Line, Shape, ShapeView, Stroke};

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2};

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

    let mut win = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let mut pip: kit::shape2d::Pipeline = r.pipeline(win.width as u32, win.height as u32);
    let mut running = true;

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let (sw, sh) = (32., 32.);
    let mut rows: u32;
    let mut cols: u32;

    // Cursor position.
    let (mut mx, mut my) = (0., 0.);

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
                        _ => {}
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        mx = position.x;
                        my = position.y;
                    }
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

        rows = (win.height as f32 / sh) as u32;
        cols = (win.width as f32 / (sw / 2.0) / 2.0) as u32;

        ///////////////////////////////////////////////////////////////////////////
        // Prepare shape view
        ///////////////////////////////////////////////////////////////////////////

        let mut sv = ShapeView::new();
        let (dx, dy) = ((mx / win.width) as f32, (my / win.height) as f32);

        for i in 0..rows {
            let y = i as f32 * sh;

            for j in 0..cols {
                let x = j as f32 * sw - sw / 2.0;

                let c1 = i as f32 / rows as f32 + dy;
                let c2 = j as f32 / cols as f32 - dx;

                if j % 2 == 0 && i % 2 == 0 {
                    sv.add(Shape::Circle(
                        Vector2::new(x + sw / 2., y + sw / 2.),
                        sw * 2.,
                        32,
                        Stroke::new(1.0, Rgba::new(0.5, c2, c1, 0.75)),
                        Fill::Empty(),
                    ));
                }

                if j * i % 2 != 0 {
                    sv.add(Shape::Rectangle(
                        Rect::new(x, y, x + sw, y + sh),
                        Stroke::new(3.0, Rgba::new(c1, c2, 0.5, 1.0)),
                        Fill::Solid(Rgba::new(1.0, dx, dy, 0.1)),
                    ));
                } else {
                    sv.add(Shape::Line(
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

        let buffer = sv.finish(&r);

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

        let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT));

        pass.set_pipeline(&pip);
        pass.set_vertex_buffer(&buffer);
        pass.draw_buffer(0..buffer.size, 0..1);
    }
}
