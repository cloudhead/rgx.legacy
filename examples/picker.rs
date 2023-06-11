use std::str::FromStr;

use rgx::gfx::prelude::*;
use rgx::ui::text::{FontFormat, FontId};

use rgx::ui::{center, hstack, painter, zstack, CursorStyle};
use rgx::ui::{Interact, WidgetExt};

pub const DEFAULT_CURSORS: &[u8] = include_bytes!("assets/cursors.rgba");
pub const DEFAULT_FONT: &[u8] = include_bytes!("assets/gohu14.uf2");

fn main() -> anyhow::Result<()> {
    let cursors = Image::try_from(DEFAULT_CURSORS).unwrap();
    let palette = [
        "#1a1c2c", "#5d275d", "#b13e53", "#ef7d57", "#ffcd75", "#a7f070", "#38b764", "#257179",
        "#29366f", "#3b5dc9", "#41a6f6", "#73eff7", "#f4f4f4", "#94b0c2", "#566c86", "#333c57",
    ];
    let swatches = palette
        .into_iter()
        .map(|s| Rgba8::from_str(s).unwrap())
        .map(|color| {
            color
                .sized((16., 16.))
                .on_click(move |_, state| *state = color)
                .boxed()
        })
        .collect::<Vec<_>>();

    let ui = zstack((
        painter(|mut canvas, color| canvas.fill(canvas.bounds(), *color)),
        center(hstack(swatches)),
    ));

    rgx::logger::init(log::Level::Debug)?;
    rgx::Application::new("picker")
        .fonts([(FontId::default(), DEFAULT_FONT, FontFormat::UF2)])?
        .cursors(cursors)
        .launch(ui, Rgba8::TRANSPARENT)
        .map_err(Into::into)
}
