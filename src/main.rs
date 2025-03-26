#![allow(unused)] // at least until everything is in a somewhat-complete state

use std::num::{NonZeroU16, NonZeroU32};
use amygui::prelude::*;
use brush::{AmyBlendModeExt, BlendEquation, BlendFactor, BlendModeA, Brush, BrushPreset, BrushPresetDraw, BrushTargetModeExt};
use layer::{Canvas, EffectTable, Layer, LayerContent, LayerTree, RasterTable};
use raylib::prelude::*;
use viewport::ViewportNode;

mod raster;
mod effect;
mod layer;
mod brush;
mod viewport;

pub struct RaylibInputBackend<'a>(pub &'a RaylibHandle);

impl InputBackend for RaylibInputBackend<'_> {
    #[inline]
    fn mouse_position(&mut self) -> Point {
        let Vector2 { x, y } = self.0.get_mouse_position();
        Point { x, y }
    }

    #[inline]
    fn is_m1_pressed(&mut self) -> bool {
        self.0.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
    }

    #[inline]
    fn is_m1_released(&mut self) -> bool {
        self.0.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
    }

    #[inline]
    fn mouse_wheel_move(&mut self) -> Point {
        let ffi::Vector2 { x, y } = self.0.get_mouse_wheel_move_v();
        Point { x, y }
    }
}

pub struct RaylibTickBackend<'a>(&'a mut RaylibHandle, &'a RaylibThread, );

impl TickBackend for RaylibTickBackend<'_> {}

pub struct RaylibDrawBackend<'a, 'b, 'c>(&'a mut RaylibDrawHandle<'b>, &'c mut RasterTable, &'c mut EffectTable, &'c mut LayerTree);

impl DrawBackend for RaylibDrawBackend<'_, '_, '_> {
    type Color = Color;

    #[inline]
    fn draw_rect(&mut self, rect: &Rect, color: &Self::Color) {
        self.0.draw_rectangle_rec(Rectangle {
            x: rect.x_min,
            y: rect.y_min,
            width: rect.width(),
            height: rect.height(),
        }, color);
    }

    #[inline]
    fn draw_text(&mut self, text: &str, top_left: Point, font_size: f32, color: &Self::Color) {
        self.0.draw_text(text, top_left.x as i32, top_left.y as i32, font_size as i32, color);
    }
}

impl_guinode_union!{
    pub enum(T) UINode<T> {
        AmyGUI(AmyGUINode<Color, T>),
        Viewport(ViewportNode),
    }
    impl(T: Node) Node;
    impl('a, T: TickNode<RaylibTickBackend<'a>>) Tick<(RaylibTickBackend<'a>)>;
    impl('a, 'b, 'c, T: DrawNode<RaylibDrawBackend<'a, 'b, 'c>>) Draw<(RaylibDrawBackend<'a, 'b, 'c>)>;
}

#[allow(clippy::cognitive_complexity)]
fn main() {
    let (mut rl, thread) = init()
        .size(1280, 720)
        .title("Amity Raster Art")
        .build();

    rl.set_exit_key(None);
    rl.set_target_fps(60);
    rl.maximize_window();

    const STYLE: ButtonStyle<Color> = ButtonStyle {
        disabled_color: Color::GRAY,
        normal_color: Color::DODGERBLUE,
        hover_color: Color::SKYBLUE,
        press_color: Color::BLUE,
    };
    let mut gui = OverlayBox::from_iter([
        UINode::Viewport(ViewportNode::new(
            Brush::new(BrushPreset::new(const { unsafe { NonZeroU16::new_unchecked(1) } }, Color::BLACK)),
            Camera2D {
                offset: Vector2::zero(),
                target: Vector2::new(-10.0, -10.0),
                rotation: 0.0,
                zoom: 1.0,
            }
        )),
        UINode::AmyGUI(AmyGUINode::PadBox(padding!(5.0, UniformGridNode::from_iter(
            24.0, 24.0,  // item size
            3.0, 3.0,    // gap
            const { unsafe { NonZeroU32::new_unchecked(2) } }, // columns
            [
                Button::new(Empty, STYLE),
                Button::new(Empty, STYLE),

                Button::new(Empty, STYLE),
                Button::new(Empty, STYLE),

                Button::new(Empty, STYLE),
                Button::new(Empty, STYLE),

                Button::new(Empty, STYLE),
                Button::new(Empty, STYLE),

                Button::new(Empty, STYLE),
                Button::new(Empty, STYLE),
            ],
        )))),
    ]);

    let mut rasters = RasterTable::new(const { unsafe { Canvas::new_unchecked(128, 128) } });
    let mut effects = EffectTable::new();
    let mut layer_tree = LayerTree::new();

    {
        let UINode::Viewport(viewport) = &mut gui.content[0] else { panic!("you forgot to update this") };
        let raster0 = rasters.create_raster(&mut rl, &thread);
        viewport.brush.set_target(raster0.clone());
        layer_tree.push(Layer::new(LayerContent::new_raster(raster0)));
    }

    while !rl.window_should_close() {
        // brush size
        if let Some(new_size) = rl.get_key_pressed()
            .map(|key| key as i32 - KeyboardKey::KEY_ONE as i32 + 1)
            .filter(|n| (1..=9).contains(n))
            .map(|n| NonZeroU16::new(u16::try_from(n).unwrap()).unwrap())
        {
            let UINode::Viewport(viewport) = &mut gui.content[0] else { panic!("you forgot to update this") };
            viewport.brush.preset.size = new_size;
        }

        let window_rec = Rect {
            x_min: 0.0,
            y_min: 0.0,
            x_max: rl.get_screen_width () as f32,
            y_max: rl.get_screen_height() as f32,
        };
        let mut ui_events = Events::check(&mut RaylibInputBackend(&rl));

        // UI must occur first because it appears in front and would consume the events
        gui.active_tick(&mut RaylibTickBackend(&mut rl, &thread), window_rec, &mut ui_events);

        // update layer buffers
        {
            for layer in layer_tree.layers_mut() {
                layer.update_buffers(&mut rl, &thread, rasters.canvas());
            }
        }

        // draw frame
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            gui.draw(&mut RaylibDrawBackend(&mut d, &mut rasters, &mut effects, &mut layer_tree), window_rec);
        }
    }
}
