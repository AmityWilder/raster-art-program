use crate::*;

pub struct Region<T> {
    pub layout: Rect,
    pub content: T,
}

impl<TB, DB, T: Node<TB, DB>> Node<TB, DB> for Region<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let width = self.layout.width();
        let height = self.layout.height();
        (
            (width, Some(width)),
            (height, Some(height)),
        )
    }

    #[inline]
    fn bounds(&self, _: Rect) -> Rect {
        self.layout
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

impl<TB, DB, T: Node<TB, DB>> ParentNode<TB, DB> for Region<T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rect) -> (&Self::Item, Rect) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rect) -> (&mut Self::Item, Rect) {
        slot = self.bounds(slot);
        (&mut self.content, slot)
    }
}
