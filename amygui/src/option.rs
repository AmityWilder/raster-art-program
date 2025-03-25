use crate::*;

impl<TB, DB, T: Node<TB, DB>> Node<TB, DB> for Option<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        match self {
            Some(inner) => inner.size_range(),
            None => ((0.0, None), (0.0, None)),
        }
    }

    #[inline]
    fn dibs_tick(&mut self, slot: Rect, events: &mut Events) {
        for (item, slot) in self.children_mut(slot) {
            item.dibs_tick(slot, events);
        }
    }

    #[inline]
    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) where TB: TickBackend {
        for (item, slot) in self.children_mut(slot) {
            if events.hover.is_some_and_overlapping(slot) {
                item.active_tick(tb, slot, events);
            } else {
                item.inactive_tick(tb, slot, events);
            }
        }
    }

    #[inline]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) where TB: TickBackend {
        for (item, slot) in self.children_mut(slot) {
            item.inactive_tick(tb, slot, events);
        }
    }

    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) where DB: DrawBackend {
        for (item, slot) in self.children(slot) {
            item.draw(d, slot);
        }
    }
}

pub struct Iter<'a, T: 'a> {
    inner: Option<&'a T>,
    slot: Rect,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = (&'a T, Rect);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = self.inner.take() {
            Some((inner, self.slot))
        } else { None }
    }
}

pub struct IterMut<'a, T: 'a> {
    inner: Option<&'a mut T>,
    slot: Rect,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = (&'a mut T, Rect);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = self.inner.take() {
            Some((inner, self.slot))
        } else { None }
    }
}

impl<TB, DB, T: Node<TB, DB>> CollectionNode<TB, DB> for Option<T> {
    type Item = T;
    type Iter<'a> = Iter<'a, T> where Self: 'a;
    type IterMut<'a> = IterMut<'a, T> where Self: 'a;

    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        Iter {
            inner: self.as_ref(),
            slot,
        }
    }

    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        IterMut {
            inner: self.as_mut(),
            slot,
        }
    }
}
