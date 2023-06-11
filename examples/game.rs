#![allow(unused_variables)]
use std::ops::ControlFlow;
use std::time;

use rgx::application::ImageOpts;
use rgx::gfx::Image;
use rgx::math::*;
use rgx::ui::canvas::Canvas;
use rgx::ui::widgets;
use rgx::ui::{Context, Env, LayoutCtx, Surfaces, Widget, WidgetEvent, WidgetLifecycle};

pub const LOGO: &[u8] = include_bytes!("assets/rx.rgba");
pub const DEFAULT_CURSORS: &[u8] = include_bytes!("assets/cursors.rgba");

struct World {}

struct Root {
    offset: Vector,
    image: widgets::Image,
    last: time::Instant,
}

impl Widget<World> for Root {
    fn update(&mut self, ctx: &Context<'_>, world: &World) {
        // self.widgets.update(delta, ctx, session);
    }

    fn layout(&mut self, parent: Size, ctx: &LayoutCtx<'_>, world: &World, env: &Env) -> Size {
        // self.widgets.layout(parent, ctx, sworld, env)
        parent
    }

    fn paint(&mut self, mut canvas: Canvas<'_>, world: &World) {
        self.image
            .paint(canvas.transform(Transform::translate(self.offset)), world);
        // self.widgets.paint(canvas.clone(), world)
    }

    fn event(
        &mut self,
        event: &WidgetEvent,
        ctx: &Context<'_>,
        world: &mut World,
    ) -> ControlFlow<()> {
        match event {
            WidgetEvent::Tick(delta) => {
                self.offset.x += 1. * (delta.as_secs_f32() * 10.);
                self.offset.y += 1. * (delta.as_secs_f32() * 10.);
            }
            _ => {}
        }
        //match event {
        //    WidgetEvent::Resized(size) => {
        //        world.handle_resize(*size);
        //    }
        //    WidgetEvent::CharacterReceived(c, mods) => {
        //        world.handle_received_character(*c, *mods);
        //    }
        //    WidgetEvent::KeyDown {
        //        key,
        //        modifiers,
        //        repeat,
        //    } => {
        //        world.handle_key_down(*key, *modifiers, *repeat);
        //    }
        //    WidgetEvent::KeyUp { key, modifiers } => {
        //        world.handle_key_up(*key, *modifiers);
        //    }
        //    WidgetEvent::MouseDown(input) => {
        //        world.handle_mouse_down(*input);
        //    }
        //    WidgetEvent::MouseUp(input) => {
        //        world.handle_mouse_up(*input);
        //    }
        //    WidgetEvent::MouseMove(point) => {
        //        world.handle_cursor_moved(*point);
        //    }
        //    WidgetEvent::Tick(delta) => {
        //        world.update(*delta);
        //    }
        //    _ => {}
        //}

        //if let flow @ ControlFlow::Break(_) = self.widgets.event(event, ctx, world) {
        //    return flow;
        //}

        //match world.tool {
        //    Tool::Pan { panning: true } => {
        //        self.cursor = CursorStyle::Grab;
        //        self.hw_cursor = "grab";
        //    }
        //    Tool::Pan { panning: false } => {
        //        self.cursor = CursorStyle::Hand;
        //        self.hw_cursor = "hand";
        //    }
        //    Tool::Brush if world.brush.is_mode(brush::Mode::Erase) => {
        //        self.cursor = CursorStyle::Pointer;
        //        self.hw_cursor = "eraser";
        //    }
        //    Tool::Brush if world.brush.is_mode(brush::Mode::Normal) => {
        //        self.cursor = CursorStyle::Pointer;
        //        self.hw_cursor = "brush";
        //    }
        //    Tool::Brush if world.brush.is_mode(brush::Mode::Pencil) => {
        //        self.cursor = CursorStyle::Pointer;
        //        self.hw_cursor = "pencil";
        //    }
        //    Tool::Sampler => {
        //        self.hw_cursor = "picker";
        //    }
        //    _ => {
        //        self.cursor = CursorStyle::Pointer;
        //    }
        //}

        //for setting in world.settings.changed() {
        //    match setting.as_str() {
        //        "ui/font" => {
        //            //
        //        }
        //        _ => {}
        //    }
        //}

        ControlFlow::Continue(())
    }

    fn contains(&self, point: Point) -> bool {
        true
        // self.widgets.contains(point)
    }

    fn lifecycle(
        &mut self,
        lifecycle: &WidgetLifecycle<'_>,
        ctx: &Context<'_>,
        world: &World,
        env: &Env,
    ) {
        self.image.lifecycle(lifecycle, ctx, world, env)
    }

    fn frame(&mut self, surfaces: &Surfaces, world: &mut World) {
        // self.widgets.frame(surfaces, world);
    }

    fn cursor(&self) -> Option<&'static str> {
        None
        // dbg!(self.widgets.hw_cursor()).or(Some(self.hw_cursor))
    }
}

fn main() -> anyhow::Result<()> {
    let ui = Root {
        last: time::Instant::now(),
        image: widgets::Image::named("logo"),
        offset: Vector::default(),
    };
    let world = World {};
    let cursors = Image::try_from(DEFAULT_CURSORS).unwrap();

    rgx::logger::init(log::Level::Debug)?;
    rgx::Application::new("game")
        .cursors(cursors)
        .image("logo", Image::try_from(LOGO)?, ImageOpts::default())
        .launch(ui, world)
        .map_err(Into::into)
}
