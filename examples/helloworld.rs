#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::sprite2d;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let size = window.inner_size().to_physical(window.hidpi_factor());

    // Setup renderer
    let mut renderer = Renderer::new(&window);

    // Setup render pipeline
    let pipeline: kit::sprite2d::Pipeline = renderer.pipeline(Blending::default());

    // Setup texture & sampler
    #[rustfmt::skip]
    let texels: [u8; 16] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    let buf = Rgba8::align(&texels);

    // Create 4 by 4 texture and sampler.
    let texture = renderer.texture(2, 2);
    let sampler = renderer.sampler(Filter::Nearest, Filter::Nearest);

    // Setup sprite
    let binding = pipeline.binding(&renderer, &texture, &sampler);
    let buffer = sprite2d::Batch::singleton(
        texture.w,
        texture.h,
        texture.rect(),
        Rect::new(0., 0., size.width as f32, size.height as f32),
        Rgba::new(0.5, 0.6, 0.8, 1.0),
        1.0,
        kit::Repeat::new(24. * (size.width / size.height) as f32, 24.),
    )
    .finish(&renderer);

    let mut textures = renderer.swap_chain(
        size.width as u32,
        size.height as u32,
        PresentMode::default(),
    );

    // Prepare resources
    renderer.submit(&[Op::Fill(&texture, buf)]);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
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
            _ => {}
        },
        Event::EventsCleared => {
            let output = textures.next();
            let mut frame = renderer.frame();

            renderer.update_pipeline(
                &pipeline,
                kit::ortho(output.width, output.height),
                &mut frame,
            );

            {
                let mut pass = frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &output);

                pass.set_pipeline(&pipeline);
                pass.draw(&buffer, &binding);
            }
            renderer.present(frame);
        }
        _ => {}
    });
}
