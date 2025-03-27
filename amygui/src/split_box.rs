use crate::*;

#[derive(Clone, Copy)]
pub struct SplitBoxLayout {
    pub direction: Direction,
    pub split_point: f32,
}

/// A UI element that displays two children sharing a variable amount of space
pub struct SplitBoxNode<T> {
    pub layout: SplitBoxLayout,
    pub content: [T; 2],
}

impl<T> SplitBoxNode<T> {
    pub const fn new(direction: Direction, split_point: f32, item1: T, item2: T) -> Self {
        Self {
            layout: SplitBoxLayout { direction, split_point },
            content: [item1, item2],
        }
    }
}

impl<T: Node> Node for SplitBoxNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let ((w_min0, w_max0), (h_min0, h_max0)) = self.content[0].size_range();
        let ((w_min1, w_max1), (h_min1, h_max1)) = self.content[1].size_range();
        let (w_max, h_max) = (w_max0.zip(w_max1), h_max0.zip(h_max1));
        match self.layout.direction {
            Direction::Row => (
                (w_min0 + w_min1, w_max.map(|(a, b)| a + b)),
                (h_min0.max(h_min1), h_max.map(|(a, b)| a.max(b))),
            ),
            Direction::Column => (
                (w_min0.max(w_min1), w_max.map(|(a, b)| a.max(b))),
                (h_min0 + h_min1, h_max.map(|(a, b)| a + b)),
            ),
        }
    }
}

pub struct Iter<'a, I> {
    iter: I,
    layout: &'a SplitBoxLayout,
    slot: std::array::IntoIter<Rect, 2>,
}

impl<'a, I> Iter<'a, I> {
    fn new(iter: I, layout: &'a SplitBoxLayout, slot: Rect) -> Self {
        let rects = match layout.direction {
            Direction::Row => {
                let (y_min, y_max) = (slot.y_min, slot.y_max);
                [
                    Rect {
                        x_min: slot.x_min,
                        y_min,
                        x_max: layout.split_point,
                        y_max,
                    },
                    Rect {
                        x_min: layout.split_point,
                        y_min,
                        x_max: slot.x_max,
                        y_max,
                    }
                ]
            }
            Direction::Column => {
                let (x_min, x_max) = (slot.x_min, slot.x_max);
                [
                    Rect {
                        x_min,
                        y_min: slot.y_min,
                        x_max,
                        y_max: layout.split_point,
                    },
                    Rect {
                        x_min,
                        y_min: layout.split_point,
                        x_max,
                        y_max: slot.y_max,
                    }
                ]
            }
        };
        Self {
            iter,
            layout,
            slot: rects.into_iter(),
        }
    }
}

impl<'a, I: Iterator> Iterator for Iter<'a, I> {
    type Item = (I::Item, Rect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().zip(self.slot.next())
    }
}

impl<'a, I: DoubleEndedIterator> DoubleEndedIterator for Iter<'a, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().zip(self.slot.next_back())
    }
}

impl<T: Node> CollectionNode for SplitBoxNode<T> {
    type Item = T;
    type Iter<'a> = Iter<'a, std::slice::Iter<'a, T>> where Self: 'a;
    type IterMut<'a> = Iter<'a, std::slice::IterMut<'a, T>> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        Iter::new(self.content.iter(), &self.layout, slot)
    }

    #[inline]
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        Iter::new(self.content.iter_mut(), &self.layout, slot)
    }
}

impl<T: Node> SimpleCollectionNode for SplitBoxNode<T> {}
