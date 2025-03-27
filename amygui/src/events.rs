use crate::*;

pub struct Event<T = ()> {
    pub(crate) event: Option<T>,
}

impl<T: Clone> Clone for Event<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { event: self.event.clone() }
    }
}
impl<T: Copy> Copy for Event<T> {}

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

    /// Same as [`Self::take`], but panics if another UI element has already claimed dibs.
    #[inline]
    pub fn take_with_dibs(&mut self) -> T {
        self.event.take().expect("only one ui element should have dibs at a time")
    }

    /// Same as [`Self::take_if`], but panics if another UI element has already claimed dibs.
    #[inline]
    pub fn take_with_dibs_if<P: FnOnce(&T) -> bool>(&mut self, predicate: P) -> Option<T> {
        if self.event.as_ref().map_or(false, predicate) {
            Some(self.take_with_dibs())
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

#[derive(Clone, Copy)]
pub struct MouseEvent {
    pub position: Point,
    pub left_mouse_press: Event<()>,
    pub scroll: Event<Point>,
}

impl Event<MouseEvent> {
    #[inline]
    pub fn is_some_and_overlapping(&self, region: Rect) -> bool {
        self.event.as_ref().is_some_and(move |e| region.contains(e.position))
    }

    #[inline]
    pub fn take_if_overlapping(&mut self, region: Rect) -> Option<MouseEvent> {
        self.take_if(move |e| region.contains(e.position))
    }
}

#[derive(Clone, Copy)]
pub struct Events {
    pub mouse_event: Event<MouseEvent>,
    /// left mouse release is not consumable, becasuse everything
    /// should be allowed to reset even if something else "consumed" it
    pub left_mouse_release: bool,
}

impl Events {
    pub fn check<IB: InputBackend>(tb: &mut IB) -> Self {
        Self {
            mouse_event: Event::new(Some(MouseEvent {
                position: tb.mouse_position(),
                left_mouse_press: Event::new(tb.is_m1_pressed().then_some(())),
                scroll: Event::new(Some(tb.mouse_wheel_move())),
            })),
            left_mouse_release: tb.is_m1_released(),
        }
    }
}
