#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use chrono::{Local, Timelike};

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Batch, Fill, Line, Rotation, Shape, Stroke};

use rgx::math::*;

use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
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

    let mut r = Renderer::new(&window);
    let mut win = window.inner_size();

    println!("{:?}", window.inner_size());

    let mut pip: kit::shape2d::Pipeline = r.pipeline(Blending::default());

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let mut textures = r.swap_chain(win.width as u32, win.height as u32, PresentMode::default());

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

                textures = r.swap_chain(w, h, PresentMode::default());

                *control_flow = ControlFlow::Poll;
            }
            WindowEvent::RedrawRequested => {
                let color = Rgba::new(0.8, 0.3, 0.3, 1.0);
                let mut batch = Batch::new();

                let x = win.width as f32 / 2.;
                let y = win.height as f32 / 2.;

                // Draw outter rim.
                batch.add(Shape::Circle(
                    Point2::new(x, y),
                    690.0,
                    1024,
                    Stroke::new(5.0, color),
                    Fill::Empty(),
                ));

                // Draw inner circle.
                batch.add(Shape::Circle(
                    Point2::new(x, y),
                    30.0,
                    1024,
                    Stroke::new(1.0, color),
                    Fill::Solid(color),
                ));

                for i in 1..(690 / 30) {
                    let mut c = color;
                    c.a = 0.2 * i as f32;

                    batch.add(Shape::Circle(
                        Point2::new(x, y),
                        30.0 * i as f32,
                        1024,
                        Stroke::new(1.0, c),
                        Fill::Empty(),
                    ));
                }

                // Draw hour ticks.
                for i in 0..12 {
                    let a = ((1.0 / 12.0) * i as f32) * std::f32::consts::PI * 2.0;

                    batch.add(Shape::Line(
                        Line::new(x, y + 690.0, x, y + 600.0),
                        Rotation::new(a, Point2::new(x, y)),
                        Stroke::new(16.0, color),
                    ));
                }

                // Draw minute ticks.
                for i in 0..60 {
                    let a = ((1.0 / 60.0) * i as f32) * std::f32::consts::PI * 2.0;

                    batch.add(Shape::Line(
                        Line::new(x, y + 690.0, x, y + 630.0),
                        Rotation::new(a, Point2::new(x, y)),
                        Stroke::new(4.0, color),
                    ));
                }

                // Draw second ticks.
                for i in 0..300 {
                    let a = ((1.0 / 300.0) * i as f32) * std::f32::consts::PI * 2.0;
                    let mut c = color;
                    c.a = 0.6;

                    batch.add(Shape::Line(
                        Line::new(x, y + 690.0, x, y + 660.0),
                        Rotation::new(a, Point2::new(x, y)),
                        Stroke::new(4.0, c),
                    ));
                }

                let now = Local::now();

                // Draw hour hand.
                {
                    let divider = 1.0 / 12.0;
                    let (_, is_hour) = now.hour12();
                    let hour = divider * is_hour as f32;
                    let minute = (divider / 60.0) * now.minute() as f32;
                    let arm_angle = (hour + minute) * std::f32::consts::PI * 2.0;

                    batch.add(Shape::Line(
                        Line::new(x, y, x, y + 420.0),
                        Rotation::new(arm_angle, Point2::new(x, y)),
                        Stroke::new(16.0, color),
                    ));
                }

                // Draw minute hand.
                {
                    let divider = 1.0 / 60.0;
                    let minute = divider * now.minute() as f32;
                    let second = (divider / 60.0) * now.second() as f32;
                    let arm_angle = (minute + second) * std::f32::consts::PI * 2.0;

                    batch.add(Shape::Line(
                        Line::new(x, y, x, y + 570.0),
                        Rotation::new(arm_angle, Point2::new(x, y)),
                        Stroke::new(8.0, color),
                    ));
                }

                // Draw second hand.
                {
                    let divider = 1.0 / 60.0;
                    let second = divider * now.second() as f32;
                    let nanosecond = (divider / 1_000_000_000 as f32) * now.nanosecond() as f32;
                    let arm_angle = (second + nanosecond) * std::f32::consts::PI * 2.0;

                    batch.add(Shape::Line(
                        Line::new(x, y, x, y + 600.0),
                        Rotation::new(arm_angle, Point2::new(x, y)),
                        Stroke::new(4.0, color),
                    ));
                }

                let buffer = batch.finish(&r);

                ///////////////////////////////////////////////////////////////////////////
                // Create frame
                ///////////////////////////////////////////////////////////////////////////

                let mut frame = r.frame();

                let out = textures.next();

                r.update_pipeline(&pip, kit::ortho(out.width, out.height), &mut frame);

                ///////////////////////////////////////////////////////////////////////////
                // Draw frame
                ///////////////////////////////////////////////////////////////////////////

                {
                    let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

                    pass.set_pipeline(&pip);
                    pass.draw_buffer(&buffer);
                }
                r.present(frame);

                *control_flow = ControlFlow::Poll;
                window.request_redraw();
            }
            _ => *control_flow = ControlFlow::Poll,
        },
        _ => *control_flow = ControlFlow::Poll,
    });
}
