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

impl<T: Node> Node for SizeBoxNode<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        (
            (self.layout.width, Some(self.layout.width)),
            (self.layout.height, Some(self.layout.height)),
        )
    }

    #[inline]
    fn bounds(&self, slot: Rectangle) -> Rectangle {
        Rectangle { x: slot.x, y: slot.y, width: self.layout.width, height: self.layout.height }
    }

    #[inline]
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<T: Node> ParentNode for SizeBoxNode<T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rectangle) -> (&T, Rectangle) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rectangle) -> (&mut T, Rectangle) {
        slot = self.bounds(slot);
        (&mut self.content, slot)
    }
}
