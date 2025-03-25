use crate::*;

#[derive(Clone, Copy)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy)]
pub struct AlignBoxLayout {
    pub horizontal: Align,
    pub vertical: Align,
}

pub struct AlignBoxNode<T> {
    pub layout: AlignBoxLayout,
    pub content: T,
}

impl<T> AlignBoxNode<T> {
    pub const fn new(horizontal: Align, vertical: Align, content: T) -> Self {
        Self {
            layout: AlignBoxLayout {
                horizontal,
                vertical,
            },
            content,
        }
    }
}

impl<T: Node> Node for AlignBoxNode<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let ((w_min, w_max), (h_min, h_max)) = self.content.size_range();
        (
            (w_min, match self.layout.horizontal { Align::Stretch => None, _ => w_max }),
            (h_min, match self.layout.vertical   { Align::Stretch => None, _ => h_max }),
        )
    }

    fn bounds(&self, slot: Rectangle) -> Rectangle {
        let (x, y, width, height);
        {
            let ((_w_min, w_max), (_h_min, h_max)) = self.content.size_range();

            match self.layout.horizontal {
                Align::Stretch => (x, width) = (slot.x, slot.width),
                Align::Start | Align::Center | Align::End => {
                    let coef = match self.layout.horizontal {
                        Align::Start  => 0.0,
                        Align::Center => 0.5,
                        Align::End    => 1.0,
                        _ => unreachable!(),
                    };
                    width = w_max.unwrap_or(slot.width);
                    x = slot.x + coef * (slot.width - width);
                }
            }

            match self.layout.vertical {
                Align::Stretch => (y, height) = (slot.y, slot.height),
                Align::Start | Align::Center | Align::End => {
                    let coef = match self.layout.vertical {
                        Align::Start  => 0.0,
                        Align::Center => 0.5,
                        Align::End    => 1.0,
                        _ => unreachable!(),
                    };
                    height = h_max.unwrap_or(slot.height);
                    y = slot.y + coef * (slot.height - height);
                }
            }
        }
        Rectangle { x, y, width, height }
    }

    #[inline]
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        let (item, slot) = self.child_mut(slot);
        item.tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        let (item, slot) = self.child(slot);
        item.draw(d, slot);
    }
}

impl<T: Node> ParentNode for AlignBoxNode<T> {
    type Item = T;

    #[inline]
    fn child(&self, slot: Rectangle) -> (&T, Rectangle) {
        (&self.content, self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rectangle) -> (&mut T, Rectangle) {
        slot = self.bounds(slot);
        (&mut self.content, slot)
    }
}
