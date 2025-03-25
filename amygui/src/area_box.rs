use crate::*;

#[derive(Clone, Copy)]
pub struct AreaBoxLayout {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

pub struct AreaBoxNode<T> {
    layout: AreaBoxLayout,
    content: T,
}

impl<TB, DB, T: Node<TB, DB>> Node<TB, DB> for AreaBoxNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let ((w_min, w_max), (h_min, h_max)) = self.content.size_range();
        let (min_width, max_width) = (self.layout.min_width, self.layout.max_width);
        let (min_height, max_height) = (self.layout.min_height, self.layout.max_height);
        let w_min = w_min.clamp(min_width, max_width);
        let h_min = h_min.clamp(min_height, max_height);
        let w_max = w_max.map_or(max_width, |w| w.clamp(min_width, max_width));
        let h_max = h_max.map_or(max_height, |h| h.clamp(min_height, max_height));
        ((w_min, Some(w_max)), (h_min, Some(h_max)))
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

impl<TB, DB, T: Node<TB, DB>> ParentNode<TB, DB> for AreaBoxNode<T> {
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
