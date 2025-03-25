use std::{cell::RefCell, rc::{Rc, Weak}};
use ide::EffectCode;
use raylib::prelude::*;

/// A built-in IDE for editing shader effects.
pub mod ide;

pub struct Effect {
    shader: Shader,
    vert_code: Option<EffectCode>,
    frag_code: Option<EffectCode>,
}

pub type RcEffect = Rc<RefCell<Effect>>;
pub type WeakEffect = Weak<RefCell<Effect>>;

impl Effect {
    #[inline]
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    #[inline]
    pub fn begin_shader_mode<'a, D: RaylibShaderModeExt>(&'a mut self, d: &'a mut D) -> RaylibShaderMode<'a, D> {
        d.begin_shader_mode(&mut self.shader)
    }
}
