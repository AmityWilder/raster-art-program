use amygui::prelude::*;
use raylib::prelude::*;
use crate::{brush::{AmyBlendModeExt, BlendEquation, BlendFactor, BlendModeA, Brush, BrushPresetDraw, BrushTargetModeExt}, RaylibDrawBackend, RaylibTickBackend};

pub struct ViewportNode {
    is_m1_space_panning: bool,
    is_m3_panning: bool,
    is_drawing: bool,
    is_cursor_shown: bool,
    brush_pos: Option<Vector2>,
    brush_pos_prev: Option<Vector2>,
    camera: Camera2D,
    pub brush: Brush,
}

impl ViewportNode {
    pub const fn new(brush: Brush, camera: Camera2D) -> Self {
        Self {
            is_m1_space_panning: false,
            is_m3_panning: false,
            is_drawing: false,
            is_cursor_shown: false,
            brush_pos: None,
            brush_pos_prev: None,
            camera,
            brush,
        }
    }
}

impl Node for ViewportNode {}

impl<'a> TickNode<RaylibTickBackend<'a>> for ViewportNode {
    fn dibs_tick(&mut self, tb: &mut RaylibTickBackend<'a>, slot: Rect, events: &mut Events) {
        // todo
    }

    fn active_tick(&mut self, tb: &mut RaylibTickBackend<'a>, slot: Rect, events: &mut Events) {
        self.brush_pos_prev = self.brush_pos;
        if let Some(mut mouse_event) = events.hover.take() {
            let RaylibTickBackend(rl, thread) = tb;

            let mouse_pos = Vector2::new(
                mouse_event.position.x,
                mouse_event.position.y,
            );

            // zoom/pan
            {
                let is_zoom_scrolling = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);

                if self.is_m1_space_panning {
                    if events.left_mouse_release || rl.is_key_released(KeyboardKey::KEY_SPACE) {
                        self.is_m1_space_panning = false
                    }
                } else {
                    if rl.is_key_down(KeyboardKey::KEY_SPACE) {
                        if mouse_event.left_mouse_press.take().is_some() {
                            self.is_m1_space_panning = true;
                        }
                    }
                }

                self.is_m3_panning = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE);

                let is_pan_scrolling = !is_zoom_scrolling;

                let mut pan = Vector2::zero();
                if is_pan_scrolling {
                    if let Some(Point { x, y }) = mouse_event.scroll.take() {
                        pan += Vector2 { x, y } * 20.0;
                    }
                }
                if self.is_m1_space_panning || self.is_m3_panning {
                    pan += rl.get_mouse_delta();
                }

                self.camera.target += (rl.get_mouse_delta() - pan) / self.camera.zoom;
                self.camera.offset = rl.get_mouse_position();

                if is_zoom_scrolling {
                    if let Some(scroll) = mouse_event.scroll.take() {
                        let scroll = if scroll.x.abs() < scroll.y.abs() { scroll.y } else { scroll.x };
                        if scroll > 0.0 {
                            if self.camera.zoom < 32.0 {
                                self.camera.zoom *= 2.0;
                            }
                        } else if scroll < 0.0 {
                            if self.camera.zoom > 0.125 {
                                self.camera.zoom /= 2.0;
                            }
                        }
                    }
                }
            }

            let mouse_world_pos = rl.get_screen_to_world2D(mouse_pos, self.camera);
            self.brush_pos = Some(mouse_world_pos);

            if self.is_drawing {
                if events.left_mouse_release {
                    self.is_drawing = false
                }
            } else {
                if mouse_event.left_mouse_press.take().is_some() {
                    self.is_drawing = true;
                }
            }

            // edit artwork
            if self.is_drawing {
                if let Some((mut d, preset)) = rl.begin_brush_target_mode(tb.1, &mut self.brush) {
                    d.draw_line_brush(preset, self.brush_pos_prev.unwrap_or(mouse_world_pos), mouse_world_pos);
                }
            }
        } else {
            self.brush_pos = None;
        }
    }

    fn inactive_tick(&mut self, tb: &mut RaylibTickBackend<'a>, slot: Rect, events: &Events) {
        self.brush_pos_prev = self.brush_pos;
    }
}

impl DrawNode<RaylibDrawBackend<'_, '_, '_>> for ViewportNode {
    fn draw(&self, d: &mut RaylibDrawBackend, slot: Rect) {
        let RaylibDrawBackend(d, rasters, effects, layer_tree) = d;

        // world
        {
            let mut d = d.begin_mode2D(self.camera);
            let px_size = self.camera.zoom.recip();

            d.draw_rectangle_rec(rasters.canvas().rec, Color::new(64,64,64,255));

            // draw artwork
            for layer in layer_tree.layers() {
                layer.draw(&mut d, rasters.canvas());
            }

            if let Some(brush_pos) = self.brush_pos {
                // brush preview
                d.draw_line_brush(&self.brush.preset, self.brush_pos_prev.unwrap_or(brush_pos), brush_pos);

                // crosshair
                {
                    const CROSSHAIR_COLOR: Color = Color::new(200,200,200,255);
                    let brush_radius = self.brush.preset.size.get() as f32 * 0.5;
                    let mut d = d.begin_blend_mode_a(BlendModeA::CustomSeparate {
                        src_rgb: BlendFactor::OneMinusDstColor,
                        dst_rgb: BlendFactor::OneMinusSrcColor,
                        src_alpha: BlendFactor::Zero,
                        dst_alpha: BlendFactor::One,
                        eq_rgb: BlendEquation::FuncAdd,
                        eq_alpha: BlendEquation::FuncAdd,
                    });
                    d.draw_ring(brush_pos, brush_radius, brush_radius + px_size, 0.0, 360.0, 20, CROSSHAIR_COLOR);
                }
            }
        }
    }
}
