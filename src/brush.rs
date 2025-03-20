use std::num::NonZeroU16;
use raylib::prelude::*;

use crate::layer::{Raster, RcRaster};

#[derive(Clone, Copy)]
pub enum BlendFactor {
    Zero                  = ffi::RL_ZERO                     as isize,
    One                   = ffi::RL_ONE                      as isize,
    SrcColor              = ffi::RL_SRC_COLOR                as isize,
    OneMinusSrcColor      = ffi::RL_ONE_MINUS_SRC_COLOR      as isize,
    SrcAlpha              = ffi::RL_SRC_ALPHA                as isize,
    OneMinusSrcAlpha      = ffi::RL_ONE_MINUS_SRC_ALPHA      as isize,
    DstAlpha              = ffi::RL_DST_ALPHA                as isize,
    OneMinusDstAlpha      = ffi::RL_ONE_MINUS_DST_ALPHA      as isize,
    DstColor              = ffi::RL_DST_COLOR                as isize,
    OneMinusDstColor      = ffi::RL_ONE_MINUS_DST_COLOR      as isize,
    SrcAlphaSaturate      = ffi::RL_SRC_ALPHA_SATURATE       as isize,
    ConstantColor         = ffi::RL_CONSTANT_COLOR           as isize,
    OneMinusConstantColor = ffi::RL_ONE_MINUS_CONSTANT_COLOR as isize,
    ConstantAlpha         = ffi::RL_CONSTANT_ALPHA           as isize,
    OneMinusConstantAlpha = ffi::RL_ONE_MINUS_CONSTANT_ALPHA as isize,
}

#[derive(Clone, Copy, Default)]
pub enum BlendEquation {
    #[default]
    FuncAdd             = ffi::RL_FUNC_ADD              as isize,
    FuncSubtract        = ffi::RL_FUNC_SUBTRACT         as isize,
    FuncReverseSubtract = ffi::RL_FUNC_REVERSE_SUBTRACT as isize,
    Min                 = ffi::RL_MIN                   as isize,
    Max                 = ffi::RL_MAX                   as isize,
}

#[derive(Clone, Copy)]
pub enum BlendModeA {
    Alpha,
    Additive,
    Multiplied,
    AddColors,
    SubtractColors,
    AlphaPremultiply,
    Custom {
        src_factor: BlendFactor,
        dst_factor: BlendFactor,
        equation:   BlendEquation,
    },
    CustomSeparate {
        src_rgb:   BlendFactor,
        dst_rgb:   BlendFactor,
        src_alpha: BlendFactor,
        dst_alpha: BlendFactor,
        eq_rgb:    BlendEquation,
        eq_alpha:  BlendEquation,
    },
}

impl Default for BlendModeA {
    fn default() -> Self {
        Self::Alpha
    }
}

pub trait AmyBlendModeExt: RaylibBlendModeExt {
    fn begin_blend_mode_a(&mut self, blend_mode: BlendModeA) -> RaylibBlendMode<'_, Self> {
        match blend_mode {
            BlendModeA::Alpha            => self.begin_blend_mode(BlendMode::BLEND_ALPHA),
            BlendModeA::Additive         => self.begin_blend_mode(BlendMode::BLEND_ADDITIVE),
            BlendModeA::Multiplied       => self.begin_blend_mode(BlendMode::BLEND_MULTIPLIED),
            BlendModeA::AddColors        => self.begin_blend_mode(BlendMode::BLEND_ADD_COLORS),
            BlendModeA::SubtractColors   => self.begin_blend_mode(BlendMode::BLEND_SUBTRACT_COLORS),
            BlendModeA::AlphaPremultiply => self.begin_blend_mode(BlendMode::BLEND_ALPHA_PREMULTIPLY),
            BlendModeA::Custom { src_factor, dst_factor, equation } => {
                unsafe { ffi::rlSetBlendFactors(src_factor as i32, dst_factor as i32, equation as i32); }
                self.begin_blend_mode(BlendMode::BLEND_CUSTOM)
            }
            BlendModeA::CustomSeparate { src_rgb, dst_rgb, src_alpha, dst_alpha, eq_rgb, eq_alpha } => {
                unsafe { ffi::rlSetBlendFactorsSeparate(src_rgb as i32, dst_rgb as i32, src_alpha as i32, dst_alpha as i32, eq_rgb as i32, eq_alpha as i32); }
                self.begin_blend_mode(BlendMode::BLEND_CUSTOM_SEPARATE)
            }
        }
    }
}
impl<D: RaylibBlendModeExt> AmyBlendModeExt for D {}

pub struct BrushPreset {
    pub size: NonZeroU16,
    pub color: Color,
    pub blend: BlendModeA,
}

impl BrushPreset {
    pub const fn new(size: NonZeroU16, color: Color) -> Self {
        Self {
            size,
            color,
            blend: BlendModeA::Alpha,
        }
    }

    pub const fn with_blend_mode(size: NonZeroU16, color: Color, blend: BlendModeA) -> Self {
        Self {
            size,
            color,
            blend,
        }
    }
}

pub trait BrushPresetDraw: RaylibDraw {
    fn draw_line_brush(&mut self, preset: &BrushPreset, p1: Vector2, p2: Vector2) {
        let thick = preset.size.get() as f32;
        let radius = thick * 0.5;
        let snapped_pos_prev = Vector2 {
            x: ((p1.x - radius).round() + radius),
            y: ((p1.y - radius).round() + radius),
        };
        let snapped_pos = Vector2 {
            x: ((p2.x - radius).round() + radius),
            y: ((p2.y - radius).round() + radius),
        };
        self.draw_line_ex(snapped_pos_prev, snapped_pos, thick, preset.color);
        self.draw_circle_v(snapped_pos_prev, radius, preset.color);
        self.draw_circle_v(snapped_pos, radius, preset.color);
    }
}
impl<T: RaylibDraw> BrushPresetDraw for T {}

pub struct Brush {
    pub preset: BrushPreset,
    target: Option<RcRaster>,
}

impl Brush {
    pub const fn new(preset: BrushPreset) -> Self {
        Self {
            preset,
            target: None,
        }
    }

    pub const fn with_target(preset: BrushPreset, target: RcRaster) -> Self {
        Self {
            preset,
            target: Some(target),
        }
    }

    pub fn set_target(&mut self, target: RcRaster) -> Option<RcRaster> {
        self.target.replace(target)
    }

    pub fn remove_target(&mut self) -> Option<RcRaster> {
        self.target.take()
    }
}

pub struct BrushTargetMode<'a, 'b, T>(&'a mut T, std::cell::RefMut<'b, RenderTexture2D>);

impl<'a, 'b, T> Drop for BrushTargetMode<'a, 'b, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndTextureMode(); }
    }
}

pub trait BrushTargetModeExt: RaylibTextureModeExt {
    #[must_use]
    fn begin_brush_target_mode<'a, 'b>(&'a mut self, _: &RaylibThread, brush: &'b mut Brush) -> Option<(BrushTargetMode<'a, 'b, Self>, &'b BrushPreset)> {
        if let Some(target_rc) = &mut brush.target {
            let target_borrow = target_rc.borrow_mut();
            unsafe { ffi::BeginTextureMode(**target_borrow); }
            Some((BrushTargetMode(self, target_borrow), &brush.preset))
        } else { None }
    }
}

impl<T: RaylibTextureModeExt> BrushTargetModeExt for T {}
impl<'a, 'b, T> RaylibDraw for BrushTargetMode<'a, 'b, T> {}
