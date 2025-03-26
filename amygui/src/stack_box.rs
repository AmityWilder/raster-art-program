use crate::*;

pub struct Iter<'a, I> {
    iter: I,
    layout: &'a StackBoxLayout,
    slot: Rect,
}

impl<'a, I> Iter<'a, I> {
    pub const fn new(iter: I, layout: &'a StackBoxLayout, slot: Rect) -> Self {
        Self {
            iter,
            layout,
            slot,
        }
    }
}

impl<'a, I: Iterator> Iterator for Iter<'a, I> {
    type Item = (I::Item, Rect);

    fn next(&mut self) -> Option<Self::Item> {
        _ = self.iter;
        _ = self.layout;
        _ = self.slot;
        todo!()
    }
}

impl<'a, I: DoubleEndedIterator> DoubleEndedIterator for Iter<'a, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct StackBoxLayout {
    pub direction: Direction,
    pub gap: f32,
}

pub struct StackBoxNode<T> {
    pub layout: StackBoxLayout,
    pub content: Vec<T>,
}

impl<T> StackBoxNode<T> {
    pub const fn new(direction: Direction, gap: f32) -> Self {
        Self::with_content(direction, gap, Vec::new())
    }

    pub const fn with_content(direction: Direction, gap: f32, content: Vec<T>) -> Self {
        Self {
            layout: StackBoxLayout { direction, gap },
            content,
        }
    }

    pub fn from_iter(direction: Direction, gap: f32, content: impl IntoIterator<Item = T>) -> Self {
        Self::with_content(direction, gap, Vec::from_iter(content))
    }
}

impl<T: Node> Node for StackBoxNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let total_gap = (self.content.len() - 1) as f32 * self.layout.gap;
        let (width, height) = match self.layout.direction {
            Direction::Row => (total_gap, 0.0),
            Direction::Column => (0.0, total_gap),
        };
        let (mut min_width, mut max_width) = (width, Some(width));
        let (mut min_height, mut max_height) = (height, Some(height));
        match self.layout.direction {
            Direction::Column => {
                for item in &self.content {
                    let ((w_min, w_max), (h_min, h_max)) = item.size_range();

                    if let Some(max) = w_max {
                        if let Some(max_width) = &mut max_width {
                            *max_width = max_width.max(max);
                        }
                    } else {
                        max_width = None;
                    }
                    min_width = min_width.max(w_min);

                    if let Some(max) = h_max {
                        if let Some(max_height) = &mut max_height {
                            *max_height += max;
                        }
                    } else {
                        max_height = None;
                    }
                    min_height += h_min;
                }
            }
            Direction::Row => {
                for item in &self.content {
                    let ((w_min, w_max), (h_min, h_max)) = item.size_range();

                    if let Some(max) = h_max {
                        if let Some(max_height) = &mut max_height {
                            *max_height = max_height.max(max);
                        }
                    } else {
                        max_height = None;
                    }
                    min_height = min_height.max(h_min);

                    if let Some(max) = w_max {
                        if let Some(max_width) = &mut max_width {
                            *max_width += max;
                        }
                    } else {
                        max_width = None;
                    }
                    min_width += w_min;
                }
            }
        }
        ((min_width, max_width), (min_height, max_height))
    }
}

impl<T: Node> CollectionNode for StackBoxNode<T> {
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

impl<T: Node> SimpleCollectionNode for StackBoxNode<T> {}
