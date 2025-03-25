use crate::*;

#[derive(Clone, Copy)]
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

pub struct Button<T = Empty> {
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

    #[inline]
    fn dibs_tick(&mut self, slot: Rectangle, events: &mut Events) {
        for (item, slot) in self.children_mut(slot) {
            item.dibs_tick(slot, events);
        }
    }

    fn active_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        if events.hover.is_some_and_overlapping(slot) {
            item.active_tick(rl, thread, slot, events);
        } else {
            item.inactive_tick(rl, thread, slot, events);
        }
        if !matches!(self.state, ButtonState::Disabled) {
            if events.left_mouse_release {
                self.state = ButtonState::Normal;
            }

            if let Some(hover) = events.hover.take_if_overlapping(slot) {
                if matches!(self.state, ButtonState::Normal) {
                    self.state = ButtonState::Hover;
                }

                if hover.left_mouse_press.is_some() {
                    self.state = ButtonState::Press;
                }
            } else {
                self.state = ButtonState::Normal;
            }
        }
    }

    #[inline]
    fn inactive_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &Events) {
        let (item, slot) = self.child_mut(slot);
        item.inactive_tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
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
