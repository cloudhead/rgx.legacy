use self::kit::shape2d;
use rgx::core::*;
use rgx::kit;

use winit::{event_loop::EventLoop, window::Window};

fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    let mut r = Renderer::new(&window)?;
    let size = window.inner_size().to_physical(window.hidpi_factor());

    let (sw, sh) = (size.width as u32, size.height as u32);
    let framebuffer = r.framebuffer(sw, sh);

    let mut textures = r.swap_chain(sw, sh, PresentMode::default());

    let pip: shape2d::Pipeline = r.pipeline(Blending::default());

    // XXX: THIS LINE CAUSES THE VALIDATION ERROR.
    r.submit(&[Op::Clear(&framebuffer, Bgra8::TRANSPARENT)]);

    let mut frame = r.frame();
    let out = textures.next();

    {
        let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);
        pass.set_pipeline(&pip);
    }

    r.present(frame);

    Ok(())
}
