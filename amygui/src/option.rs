use crate::*;

impl<T: Node> Node for Option<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        match self {
            Some(inner) => inner.size_range(),
            None => ((0.0, None), (0.0, None)),
        }
    }
}

pub struct Iter<T> {
    inner: Option<T>,
    slot: Rect,
}

impl<T> Iterator for Iter<T> {
    type Item = (T, Rect);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take().map(|x| (x, self.slot))
    }
}

impl<T> DoubleEndedIterator for Iter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next() // heeheehee mono-element iterator is symmetric :3
    }
}

impl<T: Node> CollectionNode for Option<T> {
    type Item = T;
    type Iter<'a> = Iter<&'a T> where Self: 'a;
    type IterMut<'a> = Iter<&'a mut T> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        Iter { inner: self.as_ref(), slot }
    }

    #[inline]
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        Iter { inner: self.as_mut(), slot }
    }
}

impl<T: Node> SimpleCollectionNode for Option<T> {}
