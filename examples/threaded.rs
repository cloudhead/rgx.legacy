#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::math::Point2;

use rgx::kit::shape2d;
use rgx::kit::shape2d::{Fill, Shape, Stroke};

use std::sync::{Arc, Mutex};
use std::thread;

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
    let mut size = window.inner_size().to_physical(window.hidpi_factor());

    // Setup renderer
    let mut renderer = Renderer::new(window.raw_window_handle());

    let shared_size = Arc::new(Mutex::new(size));
    let shared_coords = Arc::new(Mutex::new((0., 0.)));

    let t_shared_size = shared_size.clone();
    let t_shared_coords = shared_coords.clone();

    thread::spawn(move || {
        let (w, h) = (size.width as u32, size.height as u32);
        let mut pipeline: shape2d::Pipeline = renderer.pipeline(w, h, Blending::default());
        let mut chain = renderer.swap_chain(w, h, PresentMode::NoVsync);

        loop {
            let (w, h) = {
                let s = t_shared_size.lock().unwrap();
                (s.width as u32, s.height as u32)
            };

            if chain.width != w || chain.height != h {
                pipeline.resize(w, h);
                chain = renderer.swap_chain(w, h, PresentMode::NoVsync);
            }

            let (mx, my) = {
                *t_shared_coords.lock().unwrap()
            };
            
            let buffer = shape2d::Batch::singleton(Shape::Circle(
                Point2::new(mx, size.height as f32 - my),
                20.,
                32,
                Stroke::NONE,
                Fill::Solid(Rgba::new(1., 0., 0., 1.)),
            ))
            .finish(&renderer);

            let output = chain.next();
            let mut frame = renderer.frame();
            {
                let mut pass = frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &output);

                pass.set_pipeline(&pipeline);
                pass.draw_buffer(&buffer);
            }
            renderer.submit(frame);
        }
    });

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CursorMoved { position, .. } => {
                let mut m = shared_coords.lock().unwrap();
                m.0 = position.x as f32;
                m.1 = position.y as f32;
            }
            WindowEvent::Resized(s) => {
                size = s.to_physical(window.hidpi_factor());

                let mut shared = shared_size.lock().unwrap();
                *shared = size;
            }
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
                    *control_flow = ControlFlow::Exit;
                }
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        },
        Event::EventsCleared => {
            *control_flow = ControlFlow::Poll;
        }
        _ => {}
    });
}
