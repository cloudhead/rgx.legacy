use std::ops::ControlFlow;

use rgx::gfx::prelude::*;
use rgx::ui::text::{FontFormat, FontId, Text};

use rgx::ui::{center, hstack, Widget};
use rgx::ui::{WidgetEvent, WidgetExt};

pub const DEFAULT_CURSORS: &[u8] = include_bytes!("assets/cursors.rgba");
pub const DEFAULT_FONT: &[u8] = include_bytes!("assets/gohu14.uf2");

struct Element(Rgba8, f32, usize);

impl<T> Widget<T> for Element {
    fn paint(&mut self, mut canvas: rgx::ui::Canvas<'_>, _data: &T) {
        canvas.fill(canvas.bounds(), self.0);
        canvas.stroke(canvas.bounds(), self.1, Rgba8::WHITE);
        canvas.paint(Text::new(format!("{}", self.2)));
    }

    fn display(&self) -> String {
        format!("Element({})", self.0)
    }

    fn event(
        &mut self,
        event: &rgx::ui::WidgetEvent,
        _ctx: &rgx::ui::Context<'_>,
        _data: &mut T,
    ) -> ControlFlow<()> {
        match event {
            WidgetEvent::MouseDown(_) => {
                self.2 += 1;

                return ControlFlow::Break(());
            }
            WidgetEvent::MouseEnter(_) => {
                self.1 = 1.0;

                return ControlFlow::Break(());
            }
            WidgetEvent::MouseExit => {
                self.1 = 0.0;

                return ControlFlow::Break(());
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct State {
    clicks: u64,
    hot: bool,
}

fn main() -> anyhow::Result<()> {
    let cursors = Image::try_from(DEFAULT_CURSORS).unwrap();

    let items = vec![
        Element(Rgba8::RED, 0., 0).sized([32., 32.]).boxed(),
        Element(Rgba8::GREEN, 0., 0).sized([32., 32.]).boxed(),
        Element(Rgba8::BLUE, 0., 0).sized([32., 32.]).boxed(),
    ];
    let ui = center(hstack(items).spacing(16.));

    rgx::logger::init(log::Level::Debug)?;
    rgx::Application::new("hover")
        .fonts([(FontId::default(), DEFAULT_FONT, FontFormat::UF2)])?
        .cursors(cursors)
        .launch(ui, Rgba8::TRANSPARENT)
        .map_err(Into::into)
}
