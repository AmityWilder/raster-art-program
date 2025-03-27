use crate::*;

#[derive(Clone, Copy)]
pub struct LabelStyle<Color: Copy> {
    pub font_size: f32,
    pub color: Color,
}

pub struct Label<Color: Copy> {
    pub style: LabelStyle<Color>,
    pub content: String,
}

impl<Color: Copy> Node for Label<Color> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        todo!("measure text");
    }
}

impl<Color: Copy, TB> TickNode<TB> for Label<Color> {}

impl<Color: Copy, DB: DrawBackend<Color = Color>> DrawNode<DB> for Label<Color> {
    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        d.draw_text(&self.content, slot.min_point(), self.style.font_size, &self.style.color);
    }
}
