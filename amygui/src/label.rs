use crate::*;

#[derive(Clone, Copy)]
pub struct LabelStyle<DB: DrawBackend> {
    pub font_size: f32,
    pub color: DB::Color,
}

pub struct Label<DB: DrawBackend> {
    pub style: LabelStyle<DB>,
    pub content: String,
}

impl<TB, DB: DrawBackend> Node<TB, DB> for Label<DB> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        todo!("measure text");
    }

    fn draw(&self, d: &mut DB, slot: Rect) {
        d.draw_text(&self.content, slot.min_point(), self.style.font_size, self.style.color);
    }
}
