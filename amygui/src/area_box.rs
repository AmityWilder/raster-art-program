use crate::*;

#[derive(Clone, Copy)]
pub struct AreaBoxLayout {
    pub width:  std::ops::RangeInclusive<f32>,
    pub height: std::ops::RangeInclusive<f32>,
}

pub struct AreaBoxNode<T> {
    layout: AreaBoxLayout,
    content: T,
}

impl<T: Node> Node for AreaBoxNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let ((w_min, w_max), (h_min, h_max)) = self.content.size_range();
        let (min_width, max_width) = (*self.layout.width.start(), *self.layout.width.end());
        let (min_height, max_height) = (*self.layout.height.start(), *self.layout.height.end());
        let w_min = w_min.clamp(min_width, max_width);
        let h_min = h_min.clamp(min_height, max_height);
        let w_max = w_max.map_or(max_width, |w| w.clamp(min_width, max_width));
        let h_max = h_max.map_or(max_height, |h| h.clamp(min_height, max_height));
        ((w_min, Some(w_max)), (h_min, Some(h_max)))
    }

    #[inline]
    fn bounds(&self, slot: Rectangle) -> Rectangle {
        let Rectangle { x, y, width, height } = slot;
        // note: if min is larger than slot, the slot is being calculated wrong.
        let width = width.clamp(*self.layout.width.start(), *self.layout.width.end());
        let height = height.clamp(*self.layout.height.start(), *self.layout.height.end());
        Rectangle { x, y, width, height }
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

impl<T: Node> ParentNode for AreaBoxNode<T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rectangle) -> (&Self::Item, Rectangle) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, slot: Rectangle) -> (&mut Self::Item, Rectangle) {
        (&mut self.content, self.bounds(slot))
    }
}
