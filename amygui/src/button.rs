use crate::*;

pub struct ButtonStyle {
    pub disabled_color: Color,
    pub normal_color: Color,
    pub hover_color: Color,
    pub press_color: Color,
}

impl ButtonStyle {
    pub const DEFAULT_STYLE: Self = Self {
        disabled_color: Color::GRAY,
        normal_color: Color::DODGERBLUE,
        hover_color: Color::SKYBLUE,
        press_color: Color::BLUE,
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Disabled,
    Normal,
    Hover,
    Press,
}

pub struct Button<T = Fill> {
    pub visibility: Visibility,
    pub content: T,
    state: ButtonState,
    pub style: ButtonStyle,
}

impl<T> Button<T> {
    pub const fn new(content: T) -> Self {
        Self {
            visibility: Visibility::Occlude,
            content,
            state: ButtonState::Normal,
            style: ButtonStyle::DEFAULT_STYLE,
        }
    }

    pub const fn with_style(content: T, style: ButtonStyle) -> Self {
        Self {
            visibility: Visibility::Occlude,
            content,
            state: ButtonState::Normal,
            style,
        }
    }

    pub const fn state(&self) -> ButtonState {
        self.state
    }

    pub const fn color(&self) -> Color {
        match self.state {
            ButtonState::Disabled => self.style.disabled_color,
            ButtonState::Normal => self.style.normal_color,
            ButtonState::Hover => self.style.hover_color,
            ButtonState::Press => self.style.press_color,
        }
    }
}

impl<T: Node> Node for Button<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.content.size_range()
    }

    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.tick(rl, thread, slot, events);
        if !matches!(self.state, ButtonState::Disabled) {
            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                self.state = ButtonState::Normal;
            }
            if slot.check_collision_point_rec(rl.get_mouse_position()) {
                if matches!(self.state, ButtonState::Normal) {
                    self.state = ButtonState::Hover;
                }
            } else {
                if matches!(self.state, ButtonState::Hover) {
                    self.state = ButtonState::Normal;
                }
            }
            if matches!(self.state, ButtonState::Hover) {
                if events.left_mouse_press.is_some() {
                    events.left_mouse_press = None;
                    self.state = ButtonState::Press;
                }
            }
        }
    }

    #[inline]
    fn draw(&self, d: &mut impl RaylibDraw, slot: Rectangle) {
        d.draw_rectangle_rec(slot, self.color());
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<T: Node> ParentNode for Button<T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rectangle) -> (&T, Rectangle) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rectangle) -> (&mut T, Rectangle) {
        slot = self.bounds(slot);
        (&mut self.content, slot)
    }
}
