use crate::*;

pub struct Iter<'a> {
    layout: &'a StackBoxLayout,
    slot: Rect,
}

impl<'a> Iter<'a> {
    pub const fn new(layout: &'a StackBoxLayout, slot: Rect) -> Self {
        Self {
            layout,
            slot,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Rect;

    fn next(&mut self) -> Option<Self::Item> {
        _ = self.layout;
        _ = self.slot;
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

impl<TB, DB, T: Node<TB, DB>> Node<TB, DB> for StackBoxNode<T> {
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

impl<TB, DB, T: Node<TB, DB>> CollectionNode<TB, DB> for StackBoxNode<T> {
    type Item = T;
    type Iter<'a> = std::iter::Zip<std::slice::Iter<'a, T>, Iter<'a>> where Self: 'a;
    type IterMut<'a> = std::iter::Zip<std::slice::IterMut<'a, T>, Iter<'a>> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        self.content.iter().zip(Iter::new(&self.layout, slot))
    }

    #[inline]
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        self.content.iter_mut().zip(Iter::new(&self.layout, slot))
    }
}
