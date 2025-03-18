use std::sync::RwLock;

use raylib::prelude::*;

pub struct Layer<'a> {
    pub rtex: RwLock<RenderTexture2D>,
    pub shader: Option<&'a Shader>,
    pub children: Vec<Layer<'a>>,
}

impl<'a> Layer<'a> {
    pub fn draw_buffer<D: RaylibDraw>(&self, d: &mut D) {
        if let Some(shader) = &self.shader {
            let mut d = d.begin_shader_mode(shader);
            d.draw_texture_ex(&self.rtex, Vector2::zero(), 0.0, 1.0, Color::WHITE);
        } else {
            d.draw_texture_ex(&self.rtex, Vector2::zero(), 0.0, 1.0, Color::WHITE);
        }
    }

    pub fn update_buffers<D: RaylibTextureModeExt>(&mut self, d: &mut D, thread: &RaylibThread) {
        if !self.children.is_empty() {
            for child in &mut self.children {
                child.update_buffers(d, thread);
            }
            let mut d = d.begin_texture_mode(thread, &mut self.rtex);
            d.clear_background(Color::BLANK);
            for child in &self.children {
                child.draw_buffer(&mut d);
            }
        }
    }
}

fn main() {
    let (mut rl, thread) = init()
        .size(1280, 720)
        .title("Amity Raster Art")
        .build();
    rl.set_target_fps(60);
    rl.set_window_state(rl.get_window_state().set_window_maximized(true));

    let mut shaders: Vec<Shader> = Vec::new();
    let mut layer_tree: Vec<Layer> = Vec::new();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        for layer in &mut layer_tree {
            layer.update_buffers(&mut d, &thread);
        }

        for layer in &layer_tree {
            layer.draw_buffer(&mut d);
        }
    }
}
