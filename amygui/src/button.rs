use crate::*;

#[derive(Clone, Copy)]
pub struct ButtonStyle<ColorT: Copy> {
    pub disabled_color: ColorT,
    pub normal_color: ColorT,
    pub hover_color: ColorT,
    pub press_color: ColorT,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Disabled,
    Normal,
    Hover,
    Press,
}

pub struct Button<ColorT: Copy, T> {
    pub visibility: Visibility,
    pub content: T,
    state: ButtonState,
    pub style: ButtonStyle<ColorT>,
}

impl<ColorT: Copy, T> Button<ColorT, T> {
    pub const fn new(content: T, style: ButtonStyle<ColorT>) -> Self {
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

    pub const fn color(&self) -> &ColorT {
        match self.state {
            ButtonState::Disabled => &self.style.disabled_color,
            ButtonState::Normal => &self.style.normal_color,
            ButtonState::Hover => &self.style.hover_color,
            ButtonState::Press => &self.style.press_color,
        }
    }
}

impl<ColorT: Copy, T: Node> Node for Button<ColorT, T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.content.size_range()
    }
}

impl<ColorT: Copy, TB, T: TickNode<TB>> TickNode<TB> for Button<ColorT, T> {
    #[inline]
    fn dibs_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.dibs_tick(tb, slot, events);
    }

    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
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
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) {
        let (item, slot) = self.child_mut(slot);
        item.inactive_tick(tb, slot, events);
    }
}

impl<ColorT: Copy, DB: DrawBackend<Color = ColorT>, T: DrawNode<DB>> DrawNode<DB> for Button<ColorT, T> {
    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        d.draw_rect(&slot, self.color());
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<ColorT: Copy, T: Node> ParentNode for Button<ColorT, T> {
    type Item = T;

    #[inline(always)]
    fn content(&self) -> &Self::Item {
        &self.content
    }

    #[inline(always)]
    fn content_mut(&mut self) -> &mut Self::Item {
        &mut self.content
    }
}
