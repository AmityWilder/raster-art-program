use raylib::prelude::*;

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
