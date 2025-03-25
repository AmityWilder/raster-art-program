use crate::*;

#[derive(Clone, Copy)]
pub struct LabelStyle {
    pub font_size: i32,
    pub color: Color,
}

pub struct Label {
    pub style: LabelStyle,
    pub content: String,
}

impl Node for Label {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let height = self.style.font_size;
        let width = todo!("measure text");
        ((width, Some(width)), (height, Some(height)))
    }

    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        d.draw_text(&self.content, slot.x as i32, slot.y as i32, self.style.font_size, self.style.color);
    }
}
