use std::{ffi::CStr, num::NonZeroU16, rc::{Rc, Weak}, sync::RwLock};
use raylib::prelude::*;

pub struct Layer<'a> {
    pub rtex: Weak<RwLock<RenderTexture2D>>,
    pub shader: Option<&'a Shader>,
    pub children: Vec<Layer<'a>>,
}

impl<'a> Layer<'a> {
    pub fn draw_buffer<D: RaylibDraw>(&self, d: &mut D) {
        if let Some(rtex_rc) = self.rtex.upgrade() {
            let rtex = rtex_rc.read().expect("poison error is unrecoverable");
            if let Some(shader) = &self.shader {
                let mut d = d.begin_shader_mode(shader);
                d.draw_texture_ex(&*rtex, Vector2::zero(), 0.0, 1.0, Color::WHITE);
            } else {
                d.draw_texture_ex(&*rtex, Vector2::zero(), 0.0, 1.0, Color::WHITE);
            }
        }
    }

    pub fn update_buffers<D: RaylibTextureModeExt>(&mut self, d: &mut D, thread: &RaylibThread) {
        for child in &mut self.children {
            child.update_buffers(d, thread);
        }
        if let Some(rtex_rc) = self.rtex.upgrade() {
            let mut rtex = rtex_rc.write().expect("poison error is unrecoverable");
            let mut d = d.begin_texture_mode(thread, &mut *rtex);
            d.clear_background(Color::BLANK);
            for child in &self.children {
                child.draw_buffer(&mut d);
            }
        }
    }
}

#[allow(clippy::cognitive_complexity)]
fn main() {
    let (mut rl, thread) = init()
        .size(1280, 720)
        .title("Amity Paint")
        .build();
    rl.set_target_fps(60);
    rl.set_window_state(rl.get_window_state().set_window_maximized(true));

    let mut shaders: Vec<Shader> = Vec::new();
    let mut rasters: Vec<Rc<RwLock<RenderTexture2D>>> = Vec::new();
    let mut layer_tree: Vec<Layer> = Vec::new();
    let mut brush_size: NonZeroU16 = const { unsafe { NonZeroU16::new_unchecked(3) } };
    let mut camera = Camera2D {
        offset: Vector2::zero(),
        target: Vector2::zero(),
        rotation: 0.0,
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        let mouse_screen_pos = rl.get_mouse_position();
        let mouse_world_pos = rl.get_screen_to_world2D(mouse_screen_pos, camera);
        rl.hide_cursor();

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

        {
            let mut d = &mut rl; // `RaylibTextureModeExt` is implemented for `&mut RaylibHandle` but not `RaylibHandle`
            for layer in &mut layer_tree {
                layer.update_buffers(&mut d, &thread);
            }
        }

        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            {
                let mut d = d.begin_mode2D(camera);

                for layer in &layer_tree {
                    layer.draw_buffer(&mut d);
                }

                let ibrush_size = i32::from(brush_size.get());
                let ioffset = 1 - (brush_size.get() & 1);
                let offset = f32::from(ioffset) * 0.5;
                let ioffset = i32::from(ioffset);
                let ibrush_radius = ibrush_size/2;
                let ibrush_offset_radius = ibrush_radius + ioffset;
                let brush_radius_sqr = (ibrush_radius*ibrush_radius) as f32;
                for y in -ibrush_offset_radius..=ibrush_offset_radius {
                    let y = y as f32 + offset;
                    for x in -ibrush_offset_radius..=ibrush_offset_radius {
                        let x = x as f32 + offset;
                        if x*x + y*y <= brush_radius_sqr {
                            d.draw_rectangle_rec(Rectangle {
                                x: mouse_world_pos.x as f32 + x - 0.5,
                                y: mouse_world_pos.y as f32 + y - 0.5,
                                width: 1.0,
                                height: 1.0,
                            }, Color::WHITE);
                        }
                    }
                }
            }

            d.gui_slider_bar(Rectangle::new(0.0, 0.0, 50.0, 10.0), Some(c"0.25x"), Some(c"4x"), &mut camera.zoom, 0.25, 4.0);
        }
    }
}
