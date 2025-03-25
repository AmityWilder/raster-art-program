use crate::*;

impl<T: Node> Node for Option<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        match self {
            Some(inner) => inner.size_range(),
            None => ((0.0, None), (0.0, None)),
        }
    }

    #[inline]
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        for (item, slot) in self.children_mut(slot) {
            item.tick(rl, thread, slot, events);
        }
    }

    #[inline]
    fn draw(&self, d: &mut impl RaylibDraw, slot: Rectangle) {
        for (item, slot) in self.children(slot) {
            item.draw(d, slot);
        }
    }
}

pub struct Iter<'a, T: 'a> {
    inner: Option<&'a T>,
    slot: Rectangle,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = (&'a T, Rectangle);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = self.inner.take() {
            Some((inner, self.slot))
        } else { None }
    }
}

pub struct IterMut<'a, T: 'a> {
    inner: Option<&'a mut T>,
    slot: Rectangle,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = (&'a mut T, Rectangle);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = self.inner.take() {
            Some((inner, self.slot))
        } else { None }
    }
}

impl<T: Node> CollectionNode for Option<T> {
    type Item = T;
    type Iter<'a> = Iter<'a, T> where Self: 'a;
    type IterMut<'a> = IterMut<'a, T> where Self: 'a;

    fn children(&self, slot: Rectangle) -> Self::Iter<'_> {
        Iter {
            inner: self.as_ref(),
            slot,
        }
    }

    fn children_mut(&mut self, slot: Rectangle) -> Self::IterMut<'_> {
        IterMut {
            inner: self.as_mut(),
            slot,
        }
    }
}
