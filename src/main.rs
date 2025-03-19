use std::{cell::RefCell, num::NonZeroU16, rc::{Rc, Weak}};
use raylib::prelude::*;

type Raster = RefCell<RenderTexture2D>;
type RcRaster = Rc<Raster>;
type WeakRaster = Weak<Raster>;

type Effect = RefCell<Shader>;
type RcEffect = Rc<Effect>;
type WeakEffect = Weak<Effect>;

enum LayerContent {
    Raster {
        artwork: WeakRaster,
    },
    Group {
        buffer: RenderTexture2D,
        children: Vec<Layer>,
    },
}

struct Layer {
    content: LayerContent,
    shader: Option<WeakEffect>,
}

impl Layer {
    fn rtex<T, F: FnOnce(&RenderTexture2D) -> T>(&self, f: F) -> Option<T> {
        match &self.content {
            LayerContent::Raster { artwork, .. } => {
                if let Some(rtex_rc) = artwork.upgrade() {
                    let rtex = rtex_rc.borrow();
                    Some(f(&*rtex))
                } else { None }
            }
            LayerContent::Group { buffer, .. } => Some(f(buffer)),
        }
    }

    fn rtex_mut<T, F: FnOnce(&mut RenderTexture2D) -> T>(&mut self, f: F) -> Option<T> {
        match &mut self.content {
            LayerContent::Raster { artwork, .. } => {
                if let Some(rtex_rc) = artwork.upgrade() {
                    let mut rtex = rtex_rc.borrow_mut();
                    Some(f(&mut *rtex))
                } else { None }
            }
            LayerContent::Group { buffer, .. } => Some(f(buffer)),
        }
    }

    // this is in its own function for the purpose of recursion
    fn update_buffers<D: RaylibTextureModeExt>(&mut self, d: &mut D, thread: &RaylibThread, canvas_flipped_rec: &Rectangle, canvas_rec: &Rectangle) {
        if let LayerContent::Group { buffer, children } = &mut self.content {
            for child in &mut *children {
                child.update_buffers(d, thread, canvas_flipped_rec, canvas_rec);
            }

            {
                let mut d = d.begin_texture_mode(thread, buffer);
                d.clear_background(Color::BLANK);
                for child in &*children {
                    child.rtex(|rtex: &RenderTexture2D| {
                        if let Some(shader_rc) = child.shader.as_ref().and_then(|shader| shader.upgrade()) {
                            let shader_borrow = shader_rc.borrow();
                            let mut d = d.begin_shader_mode(&*shader_borrow);
                            d.draw_texture_pro(rtex, canvas_flipped_rec, canvas_rec, Vector2::zero(), 0.0, Color::WHITE);
                        } else {
                            d.draw_texture_pro(rtex, canvas_flipped_rec, canvas_rec, Vector2::zero(), 0.0, Color::WHITE);
                        }
                    });
                }
            }
        }
    }
}

fn create_raster<'a>(
    mut rl: &mut RaylibHandle,
    thread: &RaylibThread,
    rasters: &'a mut Vec<RcRaster>,
    w: NonZeroU16,
    h: NonZeroU16,
) -> &'a RcRaster {
    let mut rtex = rl.load_render_texture(thread, w.get().into(), h.get().into()).unwrap();
    {
        let mut d = (&mut rl).begin_texture_mode(thread, &mut rtex);
        d.clear_background(Color::BLANK);
    }
    let raster = Rc::new(RefCell::new(rtex));
    rasters.push(raster);
    let [.., last] = &rasters[..] else { unreachable!("should have at least one element after pushing") };
    last
}

fn resize_canvas(
    mut rl: &mut RaylibHandle,
    thread: &RaylibThread,
    rasters: &mut [RcRaster],
    old_w: NonZeroU16,
    old_h: NonZeroU16,
    new_w: NonZeroU16,
    new_h: NonZeroU16,
) {
    if new_w == old_w && new_h == old_h { return; }
    let old_w = old_w.get();
    let old_h = old_h.get();
    let new_w = new_w.get().into();
    let new_h = new_h.get().into();
    for raster_rc in rasters {
        let mut raster_borrow = raster_rc.borrow_mut();
        let mut new_raster = rl.load_render_texture(thread, new_w, new_h).unwrap();
        {
            let mut d = rl.begin_texture_mode(thread, &mut new_raster);
            let src_rec = Rectangle {
                x: 0.0,
                y: 0.0,
                width: old_w as f32,
                height: old_h as f32,
            };
            let dst_rec = Rectangle {
                x: 0.0,
                y: 0.0,
                width: old_w as f32,
                height: -(old_h as f32),
            };
            d.draw_texture_pro(&*raster_borrow, src_rec, dst_rec, Vector2::zero(), 0.0, Color::WHITE);
        }
        *raster_borrow = new_raster;
    }
}

fn brush_line<D: RaylibDraw>(d: &mut D, p1: Vector2, p2: Vector2, brush_size: NonZeroU16, color: Color) {
    let thick = brush_size.get() as f32;
    let radius = thick * 0.5;
    let snapped_pos_prev = Vector2 {
        x: ((p1.x - radius).round() + radius),
        y: ((p1.y - radius).round() + radius),
    };
    let snapped_pos = Vector2 {
        x: ((p2.x - radius).round() + radius),
        y: ((p2.y - radius).round() + radius),
    };
    d.draw_line_ex(snapped_pos_prev, snapped_pos, thick, color);
    d.draw_circle_v(snapped_pos_prev, radius, color);
    d.draw_circle_v(snapped_pos, radius, color);
}

#[allow(clippy::cognitive_complexity)]
fn main() {
    let (mut rl, thread) = init()
        .size(1280, 720)
        .title("Amity Raster Art")
        .build();
    rl.set_target_fps(60);
    rl.set_window_state(rl.get_window_state().set_window_maximized(true));
    let mut canvas_w = const { unsafe { NonZeroU16::new_unchecked(64) } };
    let mut canvas_h = const { unsafe { NonZeroU16::new_unchecked(64) } };
    let mut canvas_rec = Rectangle {
        x: 0.0,
        y: 0.0,
        width:  canvas_w.get() as f32,
        height: canvas_h.get() as f32,
    };
    let mut canvas_flipped_rec = canvas_rec;
    canvas_flipped_rec.height = -canvas_flipped_rec.height;

    let mut shaders: Vec<Shader> = Vec::new();
    let mut rasters: Vec<RcRaster> = Vec::new();
    let mut layer_tree: Vec<Layer> = Vec::new();
    let mut brush_size = const { unsafe { NonZeroU16::new_unchecked(3) } };
    let mut camera = Camera2D {
        offset: Vector2::zero(),
        target: Vector2::new(-10.0, -10.0),
        rotation: 0.0,
        zoom: 1.0,
    };
    let mut brush_color: Color = Color::BLACK;
    let mut brush_target: Option<RcRaster>;

    let raster0 = create_raster(&mut rl, &thread, &mut rasters, canvas_w, canvas_h);
    brush_target = Some(raster0.clone());
    layer_tree.push(Layer { content: LayerContent::Raster { artwork: Rc::downgrade(raster0) }, shader: None });

    let mut mouse_world_pos_prev = rl.get_mouse_position();

    while !rl.window_should_close() {
        let mouse_screen_pos = rl.get_mouse_position();
        rl.hide_cursor();

        // brush size
        if let Some(key) = rl.get_key_pressed() {
            match key {
                KeyboardKey::KEY_ONE   => brush_size = const { unsafe { NonZeroU16::new_unchecked(1) } },
                KeyboardKey::KEY_TWO   => brush_size = const { unsafe { NonZeroU16::new_unchecked(2) } },
                KeyboardKey::KEY_THREE => brush_size = const { unsafe { NonZeroU16::new_unchecked(3) } },
                KeyboardKey::KEY_FOUR  => brush_size = const { unsafe { NonZeroU16::new_unchecked(4) } },
                KeyboardKey::KEY_FIVE  => brush_size = const { unsafe { NonZeroU16::new_unchecked(5) } },
                KeyboardKey::KEY_SIX   => brush_size = const { unsafe { NonZeroU16::new_unchecked(6) } },
                KeyboardKey::KEY_SEVEN => brush_size = const { unsafe { NonZeroU16::new_unchecked(7) } },
                KeyboardKey::KEY_EIGHT => brush_size = const { unsafe { NonZeroU16::new_unchecked(8) } },
                KeyboardKey::KEY_NINE  => brush_size = const { unsafe { NonZeroU16::new_unchecked(9) } },
                _ => {}
            }
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
                    if camera.zoom < 8.0 {
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
            let mut d = &mut rl; // `RaylibTextureModeExt` is implemented for `&mut RaylibHandle` but not `RaylibHandle`
            if let Some(brush_target_rc) = &brush_target {
                if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let mut brush_target_borrow = brush_target_rc.borrow_mut();
                    let mut d = d.begin_texture_mode(&thread, &mut *brush_target_borrow);
                    brush_line(&mut d, mouse_world_pos_prev, mouse_world_pos, brush_size, brush_color);
                }
            }
        }

        // update layer buffers
        {
            let mut d = &mut rl; // `RaylibTextureModeExt` is implemented for `&mut RaylibHandle` but not `RaylibHandle`
            for layer in &mut layer_tree {
                layer.update_buffers(&mut d, &thread, &canvas_flipped_rec, &canvas_rec);
            }
        }

        // draw frame
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            {
                let mut d = d.begin_mode2D(camera);

                d.draw_rectangle_rec(canvas_rec, Color::new(64,64,64,255));
                // draw artwork
                for layer in &layer_tree {
                    layer.rtex(|rtex: &RenderTexture2D| {
                        if let Some(shader_rc) = layer.shader.as_ref().and_then(|shader| shader.upgrade()) {
                            let shader = shader_rc.borrow();
                            let mut d = d.begin_shader_mode(&*shader);
                            d.draw_texture_pro(rtex, canvas_flipped_rec, canvas_rec, Vector2::zero(), 0.0, Color::WHITE);
                        } else {
                            d.draw_texture_pro(rtex, canvas_flipped_rec, canvas_rec, Vector2::zero(), 0.0, Color::WHITE);
                        }
                    });
                }

                // // brush preview
                // {
                //     let ibrush_size = i32::from(brush_size.get());
                //     let ioffset = 1 - (brush_size.get() & 1);
                //     let offset = f32::from(ioffset) * 0.5;
                //     let ioffset = i32::from(ioffset);
                //     let ibrush_radius = ibrush_size/2;
                //     let ibrush_offset_radius = ibrush_radius + ioffset;
                //     let brush_radius_sqr = (ibrush_radius*ibrush_radius) as f32;
                //     for y in -ibrush_offset_radius..=ibrush_offset_radius {
                //         let y = y as f32 + offset;
                //         for x in -ibrush_offset_radius..=ibrush_offset_radius {
                //             let x = x as f32 + offset;
                //             if x*x + y*y <= brush_radius_sqr {
                //                 d.draw_rectangle_rec(Rectangle {
                //                     x: mouse_world_pos.x as f32 + x - 0.5,
                //                     y: mouse_world_pos.y as f32 + y - 0.5,
                //                     width: 1.0,
                //                     height: 1.0,
                //                 }, Color::WHITE);
                //             }
                //         }
                //     }
                // }

                brush_line(&mut d, mouse_world_pos_prev, mouse_world_pos, brush_size, Color::GRAY);

                // crosshair
                {
                    const CROSSHAIR_COLOR: Color = Color::new(200,200,200,255);
                    unsafe {
                        ffi::rlSetBlendFactorsSeparate(
                            ffi::RL_ONE_MINUS_DST_COLOR as i32, // src rgb factor
                            ffi::RL_ONE_MINUS_SRC_COLOR as i32, // dst rgb factor
                            ffi::RL_ZERO as i32, // src alpha factor
                            ffi::RL_ONE as i32, // dst alpha factor
                            ffi::RL_FUNC_ADD as i32, // rgb eq
                            ffi::RL_FUNC_ADD as i32, // alpha eq
                        );
                    }
                    let brush_radius = brush_size.get() as f32 * 0.5;
                    let px_size = camera.zoom.recip();
                    let mut d = d.begin_blend_mode(BlendMode::BLEND_CUSTOM_SEPARATE);
                    d.draw_ring(mouse_world_pos, brush_radius, brush_radius + px_size, 0.0, 360.0, 20, CROSSHAIR_COLOR);
                }
            }
        }

        mouse_world_pos_prev = mouse_world_pos;
    }
}
