#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Batch, Fill, Line, Shape, Stroke};

use rgx::math::*;

use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
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
                mx = position.x;
                my = position.y;
                window.request_redraw();
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
                let mut batch = Batch::new();

                batch.add(Shape::Circle(
                    Point2::new(win.width as f32 / 2.0, win.height as f32 / 2.0),
                    win.height as f32 / 2.0,
                    1024,
                    Stroke::new(5.0, Rgba::new(0.8, 0.3, 0.3, 1.0)),
                    Fill::Empty(),
                ));

                let x = (win.width as f32 / 2.);
                let y = (win.height as f32 / 2.);

                // batch.add(Shape::Line(
                //     Line::new(x, y, x, y - (win.height as f32 / 2.)),
                //     Stroke::new(5.0, Rgba::new(0.8, 0.3, 0.3, 1.0)),
                //     Matrix4::identity(),
                // ));

                batch.add(Shape::Line(
                    Line::new(x, y, x + (win.height as f32 / 2.), y),
                    Stroke::new(5.0, Rgba::new(1.0, 0.0, 0.0, 1.0)),
                    Matrix4::from_rotation(0.0),
                ));

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
        },
        _ => {}
    });
}
