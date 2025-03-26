use crate::*;

/// Display multiple UI layers sharing the same space.
pub struct OverlayBox<T> {
    pub content: Vec<T>,
}

impl<T> OverlayBox<T> {
    pub const fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub const fn with_content(content: Vec<T>) -> Self {
        Self { content }
    }

    pub fn from_iter(content: impl IntoIterator<Item = T>) -> Self {
        Self {
            content: Vec::from_iter(content),
        }
    }
}

impl<T: Node> Node for OverlayBox<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let mut w_min = 0.0f32; // largest min size
        let mut h_min = 0.0f32; // largest min size
        let mut w_max = Some(0.0f32); // largest max size
        let mut h_max = Some(0.0f32); // largest max size

        for child in &self.content {
            let ((w_min_child, w_max_child), (h_min_child, h_max_child)) = child.size_range();
            w_min = w_min.max(w_min_child);
            h_min = h_min.max(h_min_child);
            w_max = w_max.zip(w_max_child).map(|(a, b)| a.max(b));
            h_max = h_max.zip(h_max_child).map(|(a, b)| a.max(b));
        }

        ((w_min, w_max), (h_min, h_max))
    }
}

pub struct Iter<I> {
    iter: I,
    slot: Rect,
}

impl<I: Iterator> Iterator for Iter<I> {
    type Item = (I::Item, Rect);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| (x, self.slot))
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for Iter<I> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|x| (x, self.slot))
    }
}

impl<T: Node> CollectionNode for OverlayBox<T> {
    type Item = T;
    type Iter<'a> = Iter<std::slice::Iter<'a, T>> where Self: 'a, Self::Item: 'a;
    type IterMut<'a> = Iter<std::slice::IterMut<'a, T>> where Self: 'a, Self::Item: 'a;

    #[inline]
    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        let slot = self.bounds(slot);
        Iter { iter: self.content.iter(), slot }
    }

    #[inline]
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        let slot = self.bounds(slot);
        Iter { iter: self.content.iter_mut(), slot }
    }
}

impl<T: Node> SimpleCollectionNode for OverlayBox<T> {}
