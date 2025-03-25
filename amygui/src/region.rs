use crate::*;

pub struct Region<T> {
    pub layout: Rectangle,
    pub content: T,
}

impl<T: Node> Node for Region<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        (
            (self.layout.width, Some(self.layout.width)),
            (self.layout.height, Some(self.layout.height)),
        )
    }

    #[inline]
    fn bounds(&self, _: Rectangle) -> Rectangle {
        self.layout
    }

    #[inline]
    fn dibs_tick(&mut self, slot: Rectangle, events: &mut Events) {
        for (item, slot) in self.children_mut(slot) {
            item.dibs_tick(slot, events);
        }
    }

    #[inline]
    fn active_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        if events.hover.is_some_and_overlapping(slot) {
            item.active_tick(rl, thread, slot, events);
        } else {
            item.inactive_tick(rl, thread, slot, events);
        }
    }

    #[inline]
    fn inactive_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &Events) {
        let (item, slot) = self.child_mut(slot);
        item.inactive_tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<T: Node> ParentNode for Region<T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rectangle) -> (&Self::Item, Rectangle) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rectangle) -> (&mut Self::Item, Rectangle) {
        slot = self.bounds(slot);
        (&mut self.content, slot)
    }
}
