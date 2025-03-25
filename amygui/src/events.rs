use crate::*;

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
    pub fn take_with_dibs(&mut self) -> T {
        self.event.take().expect("only one source should have dibs at a time")
    }

    #[inline]
    pub fn take_with_dibs_if<P: FnOnce(&T) -> bool>(&mut self, predicate: P) -> Option<T> {
        if self.event.as_ref().map_or(false, predicate) {
            Some(self.event.expect("only one source should have dibs at a time"))
        } else { None }
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

pub struct MouseEvent {
    pub position: Vector2,
    pub left_mouse_press: Event<()>,
    pub scroll: Event<Vector2>,
}

impl Event<MouseEvent> {
    #[inline]
    pub fn is_some_and_overlapping(&self, region: Rectangle) -> bool {
        self.event.is_some_and(move |e| region.check_collision_point_rec(e.position))
    }

    #[inline]
    pub fn take_if_overlapping(&mut self, region: Rectangle) -> Option<MouseEvent> {
        self.take_if(move |e| region.check_collision_point_rec(e.position))
    }
}

pub struct Events {
    pub hover: Event<MouseEvent>,
    /// left mouse release is not consumable, becasuse everything
    /// should be allowed to reset even if something else "consumed" it
    pub left_mouse_release: bool,
}

impl Events {
    pub fn check(rl: &RaylibHandle) -> Self {
        Self {
            hover: Event::new(Some(MouseEvent {
                position: rl.get_mouse_position(),
                left_mouse_press: Event::new(rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT).then_some(())),
                scroll: Event::new(Some(rl.get_mouse_wheel_move_v().into())),
            })),
            left_mouse_release: rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
        }
    }
}
