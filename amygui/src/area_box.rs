use crate::*;

#[derive(Clone, Copy)]
pub struct AreaBoxLayout {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

pub struct AreaBoxNode<T> {
    layout: AreaBoxLayout,
    content: T,
}

impl<T: Node> Node for AreaBoxNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let ((w_min, w_max), (h_min, h_max)) = self.content.size_range();
        let (min_width, max_width) = (self.layout.min_width, self.layout.max_width);
        let (min_height, max_height) = (self.layout.min_height, self.layout.max_height);
        let w_min = w_min.clamp(min_width, max_width);
        let h_min = h_min.clamp(min_height, max_height);
        let w_max = w_max.map_or(max_width, |w| w.clamp(min_width, max_width));
        let h_max = h_max.map_or(max_height, |h| h.clamp(min_height, max_height));
        ((w_min, Some(w_max)), (h_min, Some(h_max)))
    }
}

impl<T: Node> ParentNode for AreaBoxNode<T> {
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

impl<T: Node> SimpleParentNode for AreaBoxNode<T> {}
