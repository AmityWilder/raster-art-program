use std::{cell::RefCell, hint::unreachable_unchecked, num::NonZeroU16, rc::{Rc, Weak}};
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

pub struct Brush {
    pub size: NonZeroU16,
    pub color: Color,
    pub blend: BlendModeA,
}

impl Brush {
    pub fn draw_line<D: RaylibDraw>(&self, d: &mut D, p1: Vector2, p2: Vector2) {
        let thick = self.size.get() as f32;
        let radius = thick * 0.5;
        let snapped_pos_prev = Vector2 {
            x: ((p1.x - radius).round() + radius),
            y: ((p1.y - radius).round() + radius),
        };
        let snapped_pos = Vector2 {
            x: ((p2.x - radius).round() + radius),
            y: ((p2.y - radius).round() + radius),
        };
        d.draw_line_ex(snapped_pos_prev, snapped_pos, thick, self.color);
        d.draw_circle_v(snapped_pos_prev, radius, self.color);
        d.draw_circle_v(snapped_pos, radius, self.color);
    }
}

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

/// **Warning:** Endless
pub struct ResizeHandleIter<'a> {
    bounds: &'a Rectangle,
    x: u8,
    y: u8,
}

impl<'a> ResizeHandleIter<'a> {
    pub fn new(bounds: &'a Rectangle) -> Self {
        Self { bounds, x: 0, y: 0 }
    }
}

impl<'a> Iterator for ResizeHandleIter<'a> {
    type Item = Vector2;

    fn next(&mut self) -> Option<Self::Item> {
        let v = Vector2 {
            x: self.bounds.x + self.bounds.width  * (0.5 * self.x as f32),
            y: self.bounds.y + self.bounds.height * (0.5 * self.y as f32),
        };
        match (self.y, self.x) {
            (0,     0 | 1) => self.x += 1,
            (0 | 1, 2    ) => self.y += 1,
            (2,     2 | 1) => self.x -= 1,
            (2 | 1, 0    ) => self.y -= 1,

            (1, 1) | (3.., _) | (_, 3..) => unreachable!("invalid iterator state"),
        }
        Some(v)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputChange {
    Released,
    Pressed,
}

pub struct InputEvents {
    mouse: [Option<InputChange>; 7],
    key: [Option<InputChange>; 336],
}

impl InputEvents {
    pub const fn new() -> Self {
        Self {
            mouse: [const { None }; 7],
            key: [const { None }; 336],
        }
    }

    pub fn update_key_event(&mut self, rl: &mut RaylibHandle, key: KeyboardKey) {
        self.key[key as i32 as usize] = rl.is_key_pressed(key).then_some(InputChange::Pressed)
            .or_else(|| rl.is_key_released(key).then_some(InputChange::Released));
    }

    pub fn update_mouse_event(&mut self, rl: &mut RaylibHandle, btn: MouseButton) {
        self.mouse[btn as i32 as usize] = rl.is_mouse_button_pressed(btn).then_some(InputChange::Pressed)
            .or_else(|| rl.is_mouse_button_released(btn).then_some(InputChange::Released));
    }

    pub fn key_event(&self, key: KeyboardKey) -> &Option<InputChange> {
        &self.key[key as i32 as usize]
    }

    pub fn mouse_event(&self, btn: MouseButton) -> &Option<InputChange> {
        &self.mouse[btn as i32 as usize]
    }

    pub fn consume_key_event(&mut self, key: KeyboardKey) {
        self.key[key as i32 as usize] = None;
    }

    pub fn consume_mouse_event(&mut self, btn: MouseButton) {
        self.mouse[btn as i32 as usize] = None;
    }
}

#[allow(clippy::cognitive_complexity)]
fn main() {
    let (mut rl, thread) = init()
        .size(1280, 720)
        .title("Amity Raster Art")
        .build();
    rl.set_exit_key(None);
    rl.set_target_fps(60);
    rl.set_window_state(rl.get_window_state().set_window_maximized(true));
    let mut canvas_w = const { unsafe { NonZeroU16::new_unchecked(128) } };
    let mut canvas_h = const { unsafe { NonZeroU16::new_unchecked(128) } };
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
    let mut camera = Camera2D {
        offset: Vector2::zero(),
        target: Vector2::new(-10.0, -10.0),
        rotation: 0.0,
        zoom: 1.0,
    };
    let mut brush_target: Option<RcRaster>;
    let mut brush = Brush {
        size: const { unsafe { NonZeroU16::new_unchecked(1) } },
        color: Color::BLACK,
        blend: BlendModeA::Alpha,
    };
    let mut input_events = InputEvents::new();

    let raster0 = create_raster(&mut rl, &thread, &mut rasters, canvas_w, canvas_h);
    brush_target = Some(raster0.clone());
    layer_tree.push(Layer { content: LayerContent::Raster { artwork: Rc::downgrade(raster0) }, shader: None });

    let mut mouse_world_pos_prev = rl.get_mouse_position();

    while !rl.window_should_close() {
        let mouse_screen_pos = rl.get_mouse_position();
        rl.hide_cursor();

        // brush size
        if let Some(new_size) = rl.get_key_pressed()
            .map(|key| key as i32 - KeyboardKey::KEY_ONE as i32 + 1)
            .filter(|n| (1..=9).contains(n))
            .map(|n| NonZeroU16::new(u16::try_from(n).unwrap()).unwrap())
        {
            brush.size = new_size;
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
            let mut d = &mut rl; // `RaylibTextureModeExt` is implemented for `&mut RaylibHandle` but not `RaylibHandle`
            if let Some(brush_target_rc) = &brush_target {
                if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let mut brush_target_borrow = brush_target_rc.borrow_mut();
                    let mut d = d.begin_texture_mode(&thread, &mut *brush_target_borrow);
                    brush.draw_line(&mut d, mouse_world_pos_prev, mouse_world_pos);
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

            // world
            {
                let mut d = d.begin_mode2D(camera);
                let px_size = camera.zoom.recip();

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

                // brush preview
                brush.draw_line(&mut d, mouse_world_pos_prev, mouse_world_pos);

                // crosshair
                {
                    const CROSSHAIR_COLOR: Color = Color::new(200,200,200,255);
                    let brush_radius = brush.size.get() as f32 * 0.5;
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

                d.draw_rectangle_rec(Rectangle::new(canvas_rec.x + canvas_rec.width + 1.0*px_size, canvas_rec.y + canvas_rec.height + 1.0*px_size, 6.0*px_size, 6.0*px_size), Color::GRAY);
                d.draw_rectangle_rec(Rectangle::new(canvas_rec.x + canvas_rec.width + 2.0*px_size, canvas_rec.y + canvas_rec.height + 2.0*px_size, 4.0*px_size, 4.0*px_size), Color::LIGHTGRAY);
            }
        }

        mouse_world_pos_prev = mouse_world_pos;
    }
}
