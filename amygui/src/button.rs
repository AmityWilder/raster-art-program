use crate::*;

#[derive(Clone, Copy)]
pub struct ButtonStyle<Color: Copy> {
    pub disabled_color: Color,
    pub normal_color: Color,
    pub hover_color: Color,
    pub press_color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Disabled,
    Normal,
    Hover,
    Press,
}

#[derive(Clone, Copy)]
pub struct ButtonData<Color: Copy> {
    pub visibility: Visibility,
    state: ButtonState,
    pub style: ButtonStyle<Color>,
}

impl<Color: Copy> ButtonData<Color> {
    pub const fn new(style: ButtonStyle<Color>) -> Self {
        Self {
            visibility: Visibility::Occlude,
            state: ButtonState::Normal,
            style,
        }
    }

    #[inline]
    pub const fn state(&self) -> ButtonState {
        self.state
    }

    #[inline]
    pub const fn color(&self) -> &Color {
        match self.state {
            ButtonState::Disabled => &self.style.disabled_color,
            ButtonState::Normal => &self.style.normal_color,
            ButtonState::Hover => &self.style.hover_color,
            ButtonState::Press => &self.style.press_color,
        }
    }
}

pub struct Button<Color: Copy, T, F> {
    pub data: ButtonData<Color>,
    pub content: T,
    pub on_press: F,
}

impl<Color: Copy, T, F> Button<Color, T, F> {
    pub const fn new(data: ButtonData<Color>, on_press: F, content: T) -> Self {
        Self {
            data,
            content,
            on_press,
        }
    }
}

impl<Color: Copy, T: Node, F> Node for Button<Color, T, F> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.content.size_range()
    }
}

impl<Color: Copy, TB, T: TickNode<TB>, F: FnMut(&mut ButtonData<Color>)> TickNode<TB> for Button<Color, T, F> {
    #[inline]
    fn dibs_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.dibs_tick(tb, slot, events);
    }

    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        {
            let (item, slot) = self.child_mut(slot);
            if events.mouse_event.is_some_and_overlapping(slot) {
                item.active_tick(tb, slot, events);
            } else {
                item.inactive_tick(tb, slot, events);
            }
        }

        if !matches!(self.data.state, ButtonState::Disabled) {
            if events.left_mouse_release {
                self.data.state = ButtonState::Hover;
            }

            if let Some(mut hover) = events.mouse_event.take_if_overlapping(slot) {
                if matches!(self.data.state, ButtonState::Normal) {
                    self.data.state = ButtonState::Hover;
                }

                if hover.left_mouse_press.take().is_some() {
                    self.data.state = ButtonState::Press;
                    (self.on_press)(&mut self.data);
                }
            } else {
                self.data.state = ButtonState::Hover;
            }
        }
    }

    #[inline]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) {
        {
            let (item, slot) = self.child_mut(slot);
            item.inactive_tick(tb, slot, events);
        }

        if !matches!(self.data.state, ButtonState::Disabled) {
            self.data.state = ButtonState::Normal;
        }
    }
}

impl<Color: Copy, DB: DrawBackend<Color = Color>, T: DrawNode<DB>, F> DrawNode<DB> for Button<Color, T, F> {
    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        d.draw_rect(&slot, self.data.color());
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<Color: Copy, T: Node, F> ParentNode for Button<Color, T, F> {
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
