use crate::*;

#[derive(Clone, Copy)]
pub struct SizeBoxLayout {
    pub width: f32,
    pub height: f32,
}

pub struct SizeBoxNode<T> {
    pub layout: SizeBoxLayout,
    pub content: T,
}

impl<T> SizeBoxNode<T> {
    pub const fn new(width: f32, height: f32, content: T) -> Self {
        Self {
            layout: SizeBoxLayout { width, height },
            content,
        }
    }
}

impl<TB, DB, T: Node<TB, DB>> Node<TB, DB> for SizeBoxNode<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        (
            (self.layout.width, Some(self.layout.width)),
            (self.layout.height, Some(self.layout.height)),
        )
    }

    #[inline]
    fn dibs_tick(&mut self, slot: Rect, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.dibs_tick(slot, events);
    }

    #[inline]
    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) where TB: TickBackend {
        let (item, slot) = self.child_mut(slot);
        if events.hover.is_some_and_overlapping(slot) {
            item.active_tick(tb, slot, events);
        } else {
            item.inactive_tick(tb, slot, events);
        }
    }

    #[inline]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) where TB: TickBackend {
        let (item, slot) = self.child_mut(slot);
        item.inactive_tick(tb, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) where DB: DrawBackend {
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<TB, DB, T: Node<TB, DB>> ParentNode<TB, DB> for SizeBoxNode<T> {
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
