use crate::*;

pub struct Iter<'a> {
    layout: &'a SplitBoxLayout,
    slot: Rect,
    counter: u8,
}

impl<'a> Iter<'a> {
    fn new(layout: &'a SplitBoxLayout, slot: Rect) -> Self {
        Self {
            layout,
            slot,
            counter: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Rect;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < 2 {
            let x_min;
            let y_min;
            let x_max;
            let y_max;
            match self.layout.direction {
                Direction::Row => {
                    (y_min, y_max) = (self.slot.y_min, self.slot.y_max);
                    if self.counter == 0 {
                        (x_min, x_max) = (self.slot.x_min, self.layout.split_point);
                    } else {
                        (x_min, x_max) = (self.layout.split_point, self.slot.x_max);
                    }
                }
                Direction::Column => {
                    (x_min, x_max) = (self.slot.x_min, self.slot.x_max);
                    if self.counter == 0 {
                        (y_min, y_max) = (self.slot.y_min, self.layout.split_point);
                    } else {
                        (y_min, y_max) = (self.layout.split_point, self.slot.y_max);
                    }

                }
            }
            self.counter += 1;
            Some(Rect { x_min, y_min, x_max, y_max })
        } else { None }
    }
}

#[derive(Clone, Copy)]
pub struct SplitBoxLayout {
    pub direction: Direction,
    pub split_point: f32,
}

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

impl<TB, DB, T: Node<TB, DB>> Node<TB, DB> for SplitBoxNode<T> {
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

impl<TB, DB, T: Node<TB, DB>> CollectionNode<TB, DB> for SplitBoxNode<T> {
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
