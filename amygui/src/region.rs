use crate::*;

pub struct Region<T> {
    pub layout: Rect,
    pub content: T,
}

impl<T: Node> Node for Region<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let width = self.layout.width();
        let height = self.layout.height();
        (
            (width, Some(width)),
            (height, Some(height)),
        )
    }
}

impl<T: Node> ParentNode for Region<T> {
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

impl<T: Node> SimpleParentNode for Region<T> {}
