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

    fn bounds(&self, slot: Rect) -> Rect {
        let (x_min, y_min, x_max, y_max);
        {
            let ((w_min, w_max), (h_min, h_max)) = self.content.size_range();

            let width = slot.width();
            let height = slot.height();
            match self.layout.horizontal {
                Align::Stretch => (x_min, x_max) = (slot.x_min, slot.x_max),
                Align::Start | Align::Center | Align::End => {
                    let t = match self.layout.horizontal {
                        Align::Start  => 0.0,
                        Align::Center => 0.5,
                        Align::End    => 1.0,
                        _ => unreachable!(),
                    };
                    todo!()
                }
            }

            match self.layout.vertical {
                Align::Stretch => (y_min, y_max) = (slot.y_min, slot.y_max),
                Align::Start | Align::Center | Align::End => {
                    let t = match self.layout.vertical {
                        Align::Start  => 0.0,
                        Align::Center => 0.5,
                        Align::End    => 1.0,
                        _ => unreachable!(),
                    };
                    todo!()
                }
            }
        }
        Rect { x_min, y_min, x_max, y_max }
    }
}

impl<T: Node> ParentNode for AlignBoxNode<T> {
    type Item = T;

    #[inline(always)]
    fn content(&self) -> &Self::Item {
        &self.content
    }

    #[inline(always)]
    fn content_mut(&mut self) -> &mut Self::Item {
        &mut self.content
    }
}

impl<T: Node> SimpleParentNode for AlignBoxNode<T> {}
