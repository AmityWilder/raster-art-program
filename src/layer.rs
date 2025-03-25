use std::{cell::RefCell, num::NonZeroU16};
use raylib::prelude::*;
use crate::{effect::{Effect, RcEffect, WeakEffect}, raster::{RcRaster, WeakRaster}};

pub enum LayerContent {
    Raster {
        artwork: WeakRaster,
    },
    Group {
        buffer: RenderTexture2D,
        children: Vec<Layer>,
    },
}

impl LayerContent {
    pub fn new_raster(artwork: &RcRaster) -> Self {
        Self::Raster {
            artwork: RcRaster::downgrade(artwork),
        }
    }

    pub const fn empty_raster() -> Self {
        Self::Raster {
            artwork: WeakRaster::new(),
        }
    }

    pub const fn new_group(buffer: RenderTexture2D) -> Self {
        Self::Group {
            buffer,
            children: Vec::new(),
        }
    }

    pub fn with_children(buffer: RenderTexture2D, children: impl IntoIterator<Item = Layer>) -> Self {
        Self::Group {
            buffer,
            children: children.into_iter().collect(),
        }
    }
}

pub struct Layer {
    pub content: LayerContent,
    pub effect: Option<WeakEffect>,
}

impl Layer {
    pub const fn new(content: LayerContent) -> Self {
        Self {
            content,
            effect: None,
        }
    }

    pub fn with_effect(content: LayerContent, effect: &RcEffect) -> Self {
        Self {
            content,
            effect: Some(RcEffect::downgrade(effect)),
        }
    }

    pub fn rtex<T, F: FnOnce(&RenderTexture2D) -> T>(&self, f: F) -> Option<T> {
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

    pub fn rtex_mut<T, F: FnOnce(&mut RenderTexture2D) -> T>(&mut self, f: F) -> Option<T> {
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
    pub fn update_buffers<D: RaylibTextureModeExt>(&mut self, d: &mut D, thread: &RaylibThread, canvas: &Canvas) {
        if let LayerContent::Group { buffer, children } = &mut self.content {
            for child in &mut *children {
                child.update_buffers(d, thread, canvas);
            }

            {
                let mut d = d.begin_texture_mode(thread, buffer);
                d.clear_background(Color::BLANK);
                for child in &*children {
                    child.rtex(|rtex: &RenderTexture2D| {
                        if let Some(effect_rc) = child.effect.as_ref().and_then(|effect| effect.upgrade()) {
                            let mut effect_borrow = effect_rc.borrow_mut();
                            let mut d = effect_borrow.begin_shader_mode(&mut d);
                            d.draw_texture_pro(rtex, canvas.flipped_rec, canvas.rec, Vector2::zero(), 0.0, Color::WHITE);
                        } else {
                            d.draw_texture_pro(rtex, canvas.flipped_rec, canvas.rec, Vector2::zero(), 0.0, Color::WHITE);
                        }
                    });
                }
            }
        }
    }

    pub fn draw<D: RaylibDraw>(&self, d: &mut D, canvas: &Canvas) {
        self.rtex(|rtex: &RenderTexture2D| {
            if let Some(effect_rc) = self.effect.as_ref().and_then(|effect| effect.upgrade()) {
                let mut effect_borrow = effect_rc.borrow_mut();
                let mut d = effect_borrow.begin_shader_mode(d);
                d.draw_texture_pro(rtex, canvas.flipped_rec, canvas.rec, Vector2::zero(), 0.0, Color::WHITE);
            } else {
                d.draw_texture_pro(rtex, canvas.flipped_rec, canvas.rec, Vector2::zero(), 0.0, Color::WHITE);
            }
        });
    }
}

#[derive(Clone, Copy)]
pub struct Canvas {
    pub w: NonZeroU16,
    pub h: NonZeroU16,
    pub rec: Rectangle,
    pub flipped_rec: Rectangle,
}

impl Canvas {
    pub const fn new(w: NonZeroU16, h: NonZeroU16) -> Self {
        let rec = Rectangle::new(0.0, 0.0, w.get() as f32, h.get() as f32);
        let mut flipped_rec = rec;
        flipped_rec.height = -flipped_rec.height;
        Self { w, h, rec, flipped_rec }
    }

    pub const unsafe fn new_unchecked(w: u16, h: u16) -> Self {
        Self::new(
            unsafe { NonZeroU16::new_unchecked(w) },
            unsafe { NonZeroU16::new_unchecked(h) },
        )
    }

    #[inline]
    pub const fn get_w(&self) -> u16 {
        self.w.get()
    }

    #[inline]
    pub const fn get_h(&self) -> u16 {
        self.h.get()
    }
}

pub struct RasterTable {
    rasters: Vec<RcRaster>,
    canvas: Canvas,
}

impl RasterTable {
    pub const fn new(canvas: Canvas) -> Self {
        Self {
            rasters: Vec::new(),
            canvas,
        }
    }

    pub const fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub const fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    pub fn create_raster<'a>(&'a mut self, mut rl: &mut RaylibHandle, thread: &RaylibThread) -> &'a RcRaster {
        let mut rtex = rl.load_render_texture(thread, self.canvas.w.get().into(), self.canvas.h.get().into()).unwrap();
        {
            let mut d = (&mut rl).begin_texture_mode(thread, &mut rtex);
            d.clear_background(Color::BLANK);
        }
        self.rasters.push(RcRaster::new(RefCell::new(rtex)));
        self.rasters.last().expect("should have at least one element after pushing")
    }

    pub fn resize_canvas(&mut self, mut rl: &mut RaylibHandle, thread: &RaylibThread, new_w: NonZeroU16, new_h: NonZeroU16) {
        if new_w == self.canvas.w && new_h == self.canvas.h { return; }
        let (inew_w, inew_h) = (new_w.get().into(), new_h.get().into());
        for raster_rc in &mut self.rasters {
            let mut raster_borrow = raster_rc.borrow_mut();
            let mut new_raster = rl.load_render_texture(thread, inew_w, inew_h).unwrap();
            {
                let mut d = rl.begin_texture_mode(thread, &mut new_raster);
                d.draw_texture_pro(&*raster_borrow, self.canvas.flipped_rec, self.canvas.rec, Vector2::zero(), 0.0, Color::WHITE);
            }
            *raster_borrow = new_raster;
        }
        self.canvas = Canvas::new(new_w, new_h);
    }

    /// Drop all unreferenced rasters
    pub fn clean(&mut self) {
        self.rasters.retain(|raster_rc| (RcRaster::strong_count(raster_rc) + RcRaster::weak_count(raster_rc)) > 1)
    }
}

pub struct EffectTable {
    effects: Vec<RcEffect>,
}

impl EffectTable {
    pub const fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn create_effect(&mut self, effect: Effect) -> &RcEffect {
        self.effects.push(RcEffect::new(RefCell::new(effect)));
        self.effects.last().expect("should have at least 1 element after pushing")
    }

    /// Drop all unreferenced effects
    pub fn clean(&mut self) {
        self.effects.retain(|effects_rc| (RcEffect::strong_count(effects_rc) + RcEffect::weak_count(effects_rc)) > 1)
    }
}

pub struct LayerTree {
    layers: Vec<Layer>,
}

impl LayerTree {
    pub const fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }

    pub fn push(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn layers(&self) -> impl IntoIterator<Item = &Layer> {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> impl IntoIterator<Item = &mut Layer> {
        &mut self.layers
    }
}
