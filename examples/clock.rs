#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use chrono::{Local, Timelike};

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Batch, Shape};

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

    let mut renderer = futures::executor::block_on(Renderer::new(&window))?;
    let mut win = window.inner_size();

    let pip: kit::shape2d::Pipeline = renderer.pipeline(Blending::default());

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let mut textures =
        renderer.swap_chain(win.width as u32, win.height as u32, PresentMode::default());

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            window.request_redraw();
            *control_flow = ControlFlow::Poll;
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
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                win = size;

                let (w, h) = (win.width as u32, win.height as u32);

                textures = renderer.swap_chain(w, h, PresentMode::default());

                *control_flow = ControlFlow::Poll;
            }
            _ => *control_flow = ControlFlow::Poll,
        },
        Event::RedrawRequested(_) => {
            let color = Rgba::new(0.8, 0.3, 0.3, 1.0);
            let mut batch = Batch::new();

            let x = win.width as f32 / 2.;
            let y = win.height as f32 / 2.;

            // Draw outter rim.
            batch.add(Shape::circle(Point2::new(x, y), 510.0, 1024).stroke(5.0, color));

            // Draw inner circles.
            batch.add(Shape::circle(Point2::new(x, y), 30.0, 1024).stroke(1.0, color));

            for i in 1..(510 / 30) {
                let mut c = color;
                c.a = 0.2 * i as f32;

                batch.add(Shape::circle(Point2::new(x, y), 30.0 * i as f32, 1024).stroke(1.0, c));
            }

            // Draw hour ticks.
            for i in 0..12 {
                let a = ((1.0 / 12.0) * i as f32) * std::f32::consts::PI * 2.0;

                batch.add(
                    Shape::line([x, y + 510.0], [x, y + 420.0])
                        .rotation(a, Point2::new(x, y))
                        .stroke(12.0, color),
                );
            }

            // Draw minute ticks.
            for i in 0..60 {
                let a = ((1.0 / 60.0) * i as f32) * std::f32::consts::PI * 2.0;

                batch.add(
                    Shape::line([x, y + 510.0], [x, y + 450.0])
                        .rotation(a, Point2::new(x, y))
                        .stroke(6.0, color),
                );
            }

            // Draw second ticks.
            for i in 0..300 {
                let a = ((1.0 / 300.0) * i as f32) * std::f32::consts::PI * 2.0;
                let mut c = color;
                c.a = 0.6;

                batch.add(
                    Shape::line([x, y + 510.0], [x, y + 480.0])
                        .rotation(a, Point2::new(x, y))
                        .stroke(2.0, c),
                );
            }

            let now = Local::now();

            // Draw hour hand.
            {
                let divider = 1.0 / 12.0;
                let (_, is_hour) = now.hour12();
                let hour = divider * is_hour as f32;
                let minute = (divider / 60.0) * now.minute() as f32;
                let arm_angle = (hour + minute) * std::f32::consts::PI * 2.0;

                batch.add(
                    Shape::line([x, y], [x, y + 240.0])
                        .rotation(arm_angle, Point2::new(x, y))
                        .stroke(12.0, color),
                );
            }

            // Draw minute hand.
            {
                let divider = 1.0 / 60.0;
                let minute = divider * now.minute() as f32;
                let second = (divider / 60.0) * now.second() as f32;
                let arm_angle = (minute + second) * std::f32::consts::PI * 2.0;

                batch.add(
                    Shape::line([x, y], [x, y + 390.0])
                        .rotation(arm_angle, Point2::new(x, y))
                        .stroke(6.0, color),
                );
            }

            // Draw second hand.
            {
                let divider = 1.0 / 60.0;
                let second = divider * now.second() as f32;
                let nanosecond = (divider / 1_000_000_000 as f32) * now.nanosecond() as f32;
                let arm_angle = (second + nanosecond) * std::f32::consts::PI * 2.0;

                batch.add(
                    Shape::line([x, y], [x, y + 420.0])
                        .rotation(arm_angle, Point2::new(x, y))
                        .stroke(2.0, color),
                );
            }

            let buffer = batch.finish(&renderer);

            ///////////////////////////////////////////////////////////////////////////
            // Create frame
            ///////////////////////////////////////////////////////////////////////////

            let mut frame = renderer.frame();

            let out = textures.next().unwrap();

            renderer.update_pipeline(
                &pip,
                kit::ortho(out.width, out.height, Default::default()),
                &mut frame,
            );

            ///////////////////////////////////////////////////////////////////////////
            // Draw frame
            ///////////////////////////////////////////////////////////////////////////

            {
                let mut pass = frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

                pass.set_pipeline(&pip);
                pass.draw_buffer(&buffer);
            }
            renderer.present(frame);

            *control_flow = ControlFlow::Poll;
            window.request_redraw();
        }
        _ => *control_flow = ControlFlow::Poll,
    });
}
