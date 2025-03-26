use crate::*;

#[derive(Clone, Copy)]
pub struct LabelStyle<ColorT: Copy> {
    pub font_size: f32,
    pub color: ColorT,
}

pub struct Label<ColorT: Copy> {
    pub style: LabelStyle<ColorT>,
    pub content: String,
}

impl<ColorT: Copy> Node for Label<ColorT> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        todo!("measure text");
    }
}

impl<ColorT: Copy, TB> TickNode<TB> for Label<ColorT> {}

impl<ColorT: Copy, DB: DrawBackend<Color = ColorT>> DrawNode<DB> for Label<ColorT> {
    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        d.draw_text(&self.content, slot.min_point(), self.style.font_size, &self.style.color);
    }
}
