use crate::*;

#[derive(Clone, Copy)]
pub struct PadBoxLayout {
    pub pad_left: f32,
    pub pad_top: f32,
    pub pad_right: f32,
    pub pad_bottom: f32,
}

pub struct PadBoxNode<T> {
    pub layout: PadBoxLayout,
    pub content: T,
}

impl<T> PadBoxNode<T> {
    pub const fn new(pad: f32, content: T) -> Self {
        Self {
            layout: PadBoxLayout {
                pad_left:   pad,
                pad_top:    pad,
                pad_right:  pad,
                pad_bottom: pad,
            },
            content,
        }
    }

    pub const fn new_vh(pad_horizontal: f32, pad_vertical: f32, content: T) -> Self {
        Self {
            layout: PadBoxLayout {
                pad_left:   pad_horizontal,
                pad_top:    pad_vertical,
                pad_right:  pad_horizontal,
                pad_bottom: pad_vertical,
            },
            content,
        }
    }

    pub const fn new_thb(pad_top: f32, pad_horizontal: f32, pad_bottom: f32, content: T) -> Self {
        Self {
            layout: PadBoxLayout {
                pad_left:   pad_horizontal,
                pad_top,
                pad_right:  pad_horizontal,
                pad_bottom,
            },
            content,
        }
    }

    pub const fn new_cw(pad_top: f32, pad_right: f32, pad_bottom: f32, pad_left: f32, content: T) -> Self {
        Self {
            layout: PadBoxLayout {
                pad_left,
                pad_top,
                pad_right,
                pad_bottom,
            },
            content,
        }
    }
}

#[macro_export]
macro_rules! padding {
    ($t:expr, $r:expr, $b:expr, $l:expr, $content:expr $(,)?) => {
        $crate::pad_box::PadBoxNode::new_cw($v, $h, $b, $l, $content)
    };

    ($t:expr, $h:expr, $b:expr, $content:expr $(,)?) => {
        $crate::pad_box::PadBoxNode::new_thb($v, $h, $b, $content)
    };

    ($v:expr, $h:expr, $content:expr $(,)?) => {
        $crate::pad_box::PadBoxNode::new_vh($v, $h, $content)
    };

    ($x:expr, $content:expr $(,)?) => {
        $crate::pad_box::PadBoxNode::new($x, $content)
    };
}

impl<T: Node> Node for PadBoxNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let ((w_min, w_max), (h_min, h_max)) = self.content.size_range();
        let pad_w = self.layout.pad_left + self.layout.pad_right;
        let pad_h = self.layout.pad_top + self.layout.pad_bottom;
        (
            (w_min + pad_w, w_max.map(|w| w + pad_w)),
            (h_min + pad_h, h_max.map(|h| h + pad_h)),
        )
    }

    #[inline]
    fn bounds(&self, slot: Rect) -> Rect {
        Rect {
            x_min: slot.x_min + self.layout.pad_left,
            y_min: slot.y_min + self.layout.pad_top,
            x_max: slot.x_max - self.layout.pad_right,
            y_max: slot.y_max - self.layout.pad_bottom,
        }
    }
}

impl<T: Node> ParentNode for PadBoxNode<T> {
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

impl<T: Node> SimpleParentNode for PadBoxNode<T> {}
