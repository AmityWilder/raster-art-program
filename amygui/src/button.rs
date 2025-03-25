use crate::*;

#[derive(Clone, Copy)]
pub struct ButtonStyle<DB: DrawBackend> {
    pub disabled_color: DB::Color,
    pub normal_color: DB::Color,
    pub hover_color: DB::Color,
    pub press_color: DB::Color,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Disabled,
    Normal,
    Hover,
    Press,
}

pub struct Button<DB: DrawBackend, T> {
    pub visibility: Visibility,
    pub content: T,
    state: ButtonState,
    pub style: ButtonStyle<DB>,
}

impl<DB: DrawBackend, T> Button<DB, T> {
    pub const fn new(content: T, style: ButtonStyle<DB>) -> Self {
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

    pub const fn color(&self) -> DB::Color {
        match self.state {
            ButtonState::Disabled => self.style.disabled_color,
            ButtonState::Normal => self.style.normal_color,
            ButtonState::Hover => self.style.hover_color,
            ButtonState::Press => self.style.press_color,
        }
    }
}

impl<TB, DB: DrawBackend, T: Node<TB, DB>> Node<TB, DB> for Button<DB, T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.content.size_range()
    }

    #[inline]
    fn dibs_tick(&mut self, slot: Rect, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.dibs_tick(slot, events);
    }

    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) where TB: TickBackend {
        let (item, slot) = self.child_mut(slot);
        if events.hover.is_some_and_overlapping(slot) {
            item.active_tick(tb, slot, events);
        } else {
            item.inactive_tick(tb, slot, events);
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
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) where TB: TickBackend {
        let (item, slot) = self.child_mut(slot);
        item.inactive_tick(tb, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        d.draw_rect(&slot, self.color());
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<TB, DB: DrawBackend, T: Node<TB, DB>> ParentNode<TB, DB> for Button<DB, T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rect) -> (&T, Rect) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rect) -> (&mut T, Rect) {
        slot = self.bounds(slot);
        (&mut self.content, slot)
    }
}
