use crate::*;

#[derive(Clone, Copy)]
pub struct SizeBoxLayout {
    pub width: f32,
    pub height: f32,
}

pub struct SizeBoxNode<T> {
    pub layout: SizeBoxLayout,
    pub content: T,
}

impl<T> SizeBoxNode<T> {
    pub const fn new(width: f32, height: f32, content: T) -> Self {
        Self {
            layout: SizeBoxLayout { width, height },
            content,
        }
    }
}

impl<T: Node> Node for SizeBoxNode<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        (
            (self.layout.width, Some(self.layout.width)),
            (self.layout.height, Some(self.layout.height)),
        )
    }
}

impl<T: Node> ParentNode for SizeBoxNode<T> {
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

impl<T: Node> SimpleParentNode for SizeBoxNode<T> {}
