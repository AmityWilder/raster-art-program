use crate::*;

pub struct Iter<'a> {
    layout: &'a SplitBoxLayout,
    slot: Rectangle,
    counter: u8,
}

impl<'a> Iter<'a> {
    fn new(layout: &'a SplitBoxLayout, slot: Rectangle) -> Self {
        Self {
            layout,
            slot,
            counter: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < 2 {
            let x;
            let y;
            let width;
            let height;
            match self.layout.direction {
                Direction::Row => {
                    (y, height) = (self.slot.y, self.slot.height);
                    if self.counter == 0 {
                        (x, width) = (self.slot.x, self.layout.split_point);
                    } else {
                        (x, width) = (self.layout.split_point, self.slot.x + self.slot.width);
                    }
                }
                Direction::Column => {
                    (x, width) = (self.slot.x, self.slot.width);
                    if self.counter == 0 {
                        (y, height) = (self.slot.y, self.layout.split_point);
                    } else {
                        (y, height) = (self.layout.split_point, self.slot.y + self.slot.height);
                    }

                }
            }
            self.counter += 1;
            Some(Rectangle { x, y, width, height })
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

    #[inline]
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        for (item, slot) in self.children_mut(slot) {
            item.tick(rl, thread, slot, events);
        }
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        for (item, slot) in self.children(slot) {
            item.draw(d, slot);
        }
    }
}

impl<T: Node> CollectionNode for SplitBoxNode<T> {
    type Item = T;
    type Iter<'a> = std::iter::Zip<std::slice::Iter<'a, T>, Iter<'a>> where Self: 'a;
    type IterMut<'a> = std::iter::Zip<std::slice::IterMut<'a, T>, Iter<'a>> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rectangle) -> Self::Iter<'_> {
        self.content.iter().zip(Iter::new(&self.layout, slot))
    }

    #[inline]
    fn children_mut(&mut self, slot: Rectangle) -> Self::IterMut<'_> {
        self.content.iter_mut().zip(Iter::new(&self.layout, slot))
    }
}
