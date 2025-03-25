#![allow(unused)] // at least until everything is in a somewhat-complete state

use std::num::NonZeroU16;
use amygui::{button::{Button, ButtonState, ButtonStyle}, padding, size_box::{SizeBoxLayout, SizeBoxNode}, CollectionNode, Events, Fill, Node, ParentNode, Visibility};
use brush::{AmyBlendModeExt, BlendEquation, BlendFactor, BlendModeA, Brush, BrushPreset, BrushPresetDraw, BrushTargetModeExt};
use events::{Input, InputEvents};
use layer::{Canvas, EffectTable, Layer, LayerContent, LayerTree, RasterTable};
use raylib::prelude::*;

mod raster;
mod effect;
mod events;
mod layer;
mod brush;

#[allow(clippy::cognitive_complexity)]
fn main() {
    let (mut rl, thread) = init()
        .size(1280, 720)
        .title("Amity Raster Art")
        .build();
    rl.set_exit_key(None);
    rl.set_target_fps(60);
    rl.set_window_state(rl.get_window_state().set_window_maximized(true));

    let mut rasters = RasterTable::new(const { unsafe { Canvas::new_unchecked(128, 128) } });
    let mut effects = EffectTable::new();
    let mut layer_tree = LayerTree::new();
    let mut camera = Camera2D {
        offset: Vector2::zero(),
        target: Vector2::new(-10.0, -10.0),
        rotation: 0.0,
        zoom: 1.0,
    };
    let mut input_events = InputEvents::new();
    let mut brush = Brush::new(BrushPreset::new(const { unsafe { NonZeroU16::new_unchecked(1) } }, Color::BLACK));

    {
        let raster0 = rasters.create_raster(&mut rl, &thread);
        brush.set_target(raster0.clone());
        layer_tree.push(Layer::new(LayerContent::new_raster(raster0)));
    }

    let mut mouse_world_pos_prev = rl.get_mouse_position();

    let mut ui_test_box = SizeBoxNode {
        layout: SizeBoxLayout {
            width: 50.0,
            height: 50.0,
        },
        content: Button::new(padding!(
            20.0,
            Button::new(Fill),
        )),
    };

    while !rl.window_should_close() {
        input_events.check(&rl, [
            Input::from(MouseButton::MOUSE_BUTTON_LEFT),
            Input::from(MouseButton::MOUSE_BUTTON_RIGHT),
            Input::from(MouseButton::MOUSE_BUTTON_MIDDLE),
            Input::from(KeyboardKey::KEY_SPACE),
            Input::from(KeyboardKey::KEY_ONE),
            Input::from(KeyboardKey::KEY_TWO),
            Input::from(KeyboardKey::KEY_THREE),
            Input::from(KeyboardKey::KEY_FOUR),
            Input::from(KeyboardKey::KEY_FIVE),
            Input::from(KeyboardKey::KEY_SIX),
            Input::from(KeyboardKey::KEY_SEVEN),
            Input::from(KeyboardKey::KEY_EIGHT),
            Input::from(KeyboardKey::KEY_NINE),
            Input::from(KeyboardKey::KEY_LEFT_SHIFT),
            Input::from(KeyboardKey::KEY_LEFT_CONTROL),
        ]);
        let mouse_screen_pos = rl.get_mouse_position();
        // rl.hide_cursor();

        let window_rec = Rectangle {
            x: 0.0,
            y: 0.0,
            width: rl.get_screen_width() as f32,
            height: rl.get_screen_height() as f32,
        };
        let mut ui_events = Events::check(&rl);
        ui_test_box.tick(&mut rl, &thread, window_rec, &mut ui_events);

        // brush size
        if let Some(new_size) = rl.get_key_pressed()
            .map(|key| key as i32 - KeyboardKey::KEY_ONE as i32 + 1)
            .filter(|n| (1..=9).contains(n))
            .map(|n| NonZeroU16::new(u16::try_from(n).unwrap()).unwrap())
        {
            brush.preset.size = new_size;
        }

        // zoom/pan
        {
            let is_zoom_scrolling = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);
            let is_pan_dragging = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) ||
                (rl.is_key_down(KeyboardKey::KEY_SPACE) && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT));

            let is_pan_scrolling = !is_zoom_scrolling;

            let mut pan = Vector2::zero();
            if is_pan_scrolling {
                pan += Vector2::from(rl.get_mouse_wheel_move_v()) * 20.0;
            }
            if is_pan_dragging {
                pan += rl.get_mouse_delta();
            }

            camera.target += (rl.get_mouse_delta() - pan) / camera.zoom;
            camera.offset = mouse_screen_pos;

            if is_zoom_scrolling {
                let scroll = rl.get_mouse_wheel_move();
                if scroll > 0.0 {
                    if camera.zoom < 32.0 {
                        camera.zoom *= 2.0;
                    }
                } else if scroll < 0.0 {
                    if camera.zoom > 0.125 {
                        camera.zoom /= 2.0;
                    }
                }
            }
        }

        let mouse_world_pos = rl.get_screen_to_world2D(mouse_screen_pos, camera);

        // edit artwork
        {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                if let Some((mut d, preset)) = (&mut &mut rl).begin_brush_target_mode(&thread, &mut brush) {
                    d.draw_line_brush(preset, mouse_world_pos_prev, mouse_world_pos);
                }
            }
        }

        // update layer buffers
        {
            let mut d = &mut rl;
            for layer in layer_tree.layers_mut() {
                layer.update_buffers(&mut d, &thread, rasters.canvas());
            }
        }

        // draw frame
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            // world
            {
                let mut d = d.begin_mode2D(camera);
                let px_size = camera.zoom.recip();

                d.draw_rectangle_rec(rasters.canvas().rec, Color::new(64,64,64,255));

                // draw artwork
                for layer in layer_tree.layers() {
                    layer.draw(&mut d, rasters.canvas());
                }

                // brush preview
                d.draw_line_brush(&brush.preset, mouse_world_pos_prev, mouse_world_pos);

                // crosshair
                {
                    const CROSSHAIR_COLOR: Color = Color::new(200,200,200,255);
                    let brush_radius = brush.preset.size.get() as f32 * 0.5;
                    let mut d = d.begin_blend_mode_a(BlendModeA::CustomSeparate {
                        src_rgb: BlendFactor::OneMinusDstColor,
                        dst_rgb: BlendFactor::OneMinusSrcColor,
                        src_alpha: BlendFactor::Zero,
                        dst_alpha: BlendFactor::One,
                        eq_rgb: BlendEquation::FuncAdd,
                        eq_alpha: BlendEquation::FuncAdd,
                    });
                    d.draw_ring(mouse_world_pos, brush_radius, brush_radius + px_size, 0.0, 360.0, 20, CROSSHAIR_COLOR);
                }
            }

            ui_test_box.draw(&mut d, window_rec);
        }

        mouse_world_pos_prev = mouse_world_pos;
    }
}
