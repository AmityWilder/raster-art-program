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
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut impl RaylibDraw, slot: Rectangle) {
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
