use crate::*;

pub struct Dibs(());

pub struct Event<T = ()> {
    event: Option<T>,
}

impl<T> Event<T> {
    fn new(event: Option<T>) -> Self {
        Self {
            event,
        }
    }

    #[inline]
    pub fn take(&mut self) -> Option<T> {
        self.event.take()
    }

    #[inline]
    pub fn take_if<P: FnOnce(&T) -> bool>(&mut self, predicate: P) -> Option<T> {
        self.event.take_if(|x| predicate(x))
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        self.event.is_some()
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.event.is_none()
    }
}

pub struct Events {
    pub hover: Event<Vector2>,
    pub left_mouse_press: Event<()>,
    /// left mouse release is not consumable, becasuse everything
    /// should be allowed to reset even if something else "consumed" it
    pub left_mouse_release: bool,
    pub scroll: Event<Vector2>,
}

impl Events {
    pub fn check(rl: &RaylibHandle) -> Self {
        Self {
            hover: Event::new(Some(rl.get_mouse_position())),
            left_mouse_press: Event::new(rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT).then_some(())),
            left_mouse_release: rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
            scroll: Event::new(Some(rl.get_mouse_wheel_move_v().into())),
        }
    }
}
