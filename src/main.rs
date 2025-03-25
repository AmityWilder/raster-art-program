#![allow(unused)] // at least until everything is in a somewhat-complete state

use std::num::{NonZeroU16, NonZeroU32};
use amygui::{button::{Button, ButtonState, ButtonStyle}, events::Events, padding, size_box::{SizeBoxLayout, SizeBoxNode}, uniform_grid::UniformGridNode, CollectionNode, Fill, Node, ParentNode, Visibility};
use brush::{AmyBlendModeExt, BlendEquation, BlendFactor, BlendModeA, Brush, BrushPreset, BrushPresetDraw, BrushTargetModeExt};
use layer::{Canvas, EffectTable, Layer, LayerContent, LayerTree, RasterTable};
use raylib::prelude::*;

mod raster;
mod effect;
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
    rl.maximize_window();

    let mut rasters = RasterTable::new(const { unsafe { Canvas::new_unchecked(128, 128) } });
    let mut effects = EffectTable::new();
    let mut layer_tree = LayerTree::new();
    let mut camera = Camera2D {
        offset: Vector2::zero(),
        target: Vector2::new(-10.0, -10.0),
        rotation: 0.0,
        zoom: 1.0,
    };
    let mut brush = Brush::new(BrushPreset::new(const { unsafe { NonZeroU16::new_unchecked(1) } }, Color::BLACK));

    {
        let raster0 = rasters.create_raster(&mut rl, &thread);
        brush.set_target(raster0.clone());
        layer_tree.push(Layer::new(LayerContent::new_raster(raster0)));
    }

    let mut mouse_world_pos_prev = None;

    let mut tool_panel = padding!(5.0, UniformGridNode::from_iter(
        rvec2(24.0, 24.0),  // item size
        rvec2(3.0, 3.0),    // gap
        const { unsafe { NonZeroU32::new_unchecked(2) } }, // columns
        [
            Button::new(Fill),
            Button::new(Fill),

            Button::new(Fill),
            Button::new(Fill),

            Button::new(Fill),
            Button::new(Fill),

            Button::new(Fill),
            Button::new(Fill),

            Button::new(Fill),
            Button::new(Fill),
        ],
    ));

    let mut is_m1_space_panning = false;
    let mut is_m3_panning = false;
    let mut is_drawing = false;
    let mut is_cursor_shown = true;

    while !rl.window_should_close() {
        let window_rec = Rectangle {
            x: 0.0,
            y: 0.0,
            width: rl.get_screen_width() as f32,
            height: rl.get_screen_height() as f32,
        };
        let mut ui_events = Events::check(&rl);

        // UI must occur first because it appears in front and would consume the events
        tool_panel.tick(&mut rl, &thread, window_rec, &mut ui_events);

        // show cursor while hovering UI
        if ui_events.hover.is_none() != is_cursor_shown {
            is_cursor_shown = !is_cursor_shown;
            if is_cursor_shown {
                rl.show_cursor();
            } else {
                rl.hide_cursor();
            }
        }

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

            if is_m1_space_panning {
                if ui_events.left_mouse_release || rl.is_key_released(KeyboardKey::KEY_SPACE) {
                    is_m1_space_panning = false
                }
            } else {
                if rl.is_key_down(KeyboardKey::KEY_SPACE) {
                    if ui_events.left_mouse_press.take().is_some() {
                        is_m1_space_panning = true;
                    }
                }
            }

            is_m3_panning = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE);

            let is_pan_scrolling = !is_zoom_scrolling;

            let mut pan = Vector2::zero();
            if is_pan_scrolling {
                if let Some(scroll) = ui_events.scroll.take() {
                    pan += scroll * 20.0;
                }
            }
            if is_m1_space_panning || is_m3_panning {
                pan += rl.get_mouse_delta();
            }

            camera.target += (rl.get_mouse_delta() - pan) / camera.zoom;
            camera.offset = rl.get_mouse_position();

            if is_zoom_scrolling {
                if let Some(scroll) = ui_events.scroll.take() {
                    let scroll = if scroll.x.abs() < scroll.y.abs() { scroll.y } else { scroll.x };
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
        }

        let mouse_world_pos = ui_events.hover.take().map(|pos| rl.get_screen_to_world2D(pos, camera));
        if let Some(mouse_screen_pos) = ui_events.hover.take() {}

        if is_drawing {
            if ui_events.left_mouse_release {
                is_drawing = false
            }
        } else {
            if ui_events.left_mouse_press.take().is_some() {
                is_drawing = true;
            }
        }

        // edit artwork
        if let Some(mouse_world_pos) = mouse_world_pos {
            if is_drawing {
                if let Some((mut d, preset)) = rl.begin_brush_target_mode(&thread, &mut brush) {
                    d.draw_line_brush(preset, mouse_world_pos_prev.unwrap_or(mouse_world_pos), mouse_world_pos);
                }
            }
        }

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

            // world
            {
                let mut d = d.begin_mode2D(camera);
                let px_size = camera.zoom.recip();

                d.draw_rectangle_rec(rasters.canvas().rec, Color::new(64,64,64,255));

                // draw artwork
                for layer in layer_tree.layers() {
                    layer.draw(&mut d, rasters.canvas());
                }

                if let Some(mouse_world_pos) = mouse_world_pos {
                    // brush preview
                    d.draw_line_brush(&brush.preset, mouse_world_pos_prev.unwrap_or(mouse_world_pos), mouse_world_pos);

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
            }

            // ui is drawn last because it appears in front
            tool_panel.draw(&mut d, window_rec);
        }

        mouse_world_pos_prev = mouse_world_pos;
    }
}
