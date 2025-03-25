use std::num::NonZeroU32;
use crate::*;

pub struct Iter<'a> {
    layout: &'a UniformGridLayout,
    index: u32,
    src_xy: Vector2,
}

impl<'a> Iter<'a> {
    pub const fn new(layout: &'a UniformGridLayout, slot: Rectangle) -> Self {
        Self {
            layout,
            index: 0,
            src_xy: Vector2 { x: slot.x, y: slot.y },
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Self::Item> {
        let Vector2 { x: width, y: height } = self.layout.item_size;
        let index = self.index;
        self.index += 1;
        let (row, col) = (
            index / self.layout.num_columns,
            index % self.layout.num_columns,
        );
        Some(Rectangle {
            x: self.src_xy.x + col as f32 * (width + self.layout.column_gap),
            y: self.src_xy.y + row as f32 * (height + self.layout.row_gap),
            width,
            height,
        })
    }
}

#[derive(Clone, Copy)]
pub struct UniformGridLayout {
    pub item_size: Vector2,
    pub row_gap: f32,
    pub column_gap: f32,
    pub num_columns: NonZeroU32,
}

pub struct UniformGridNode<T> {
    pub layout: UniformGridLayout,
    pub content: Vec<T>,
}

impl<T> UniformGridNode<T> {
    pub const fn new(item_size: Vector2, gap: Vector2, num_columns: NonZeroU32) -> Self {
        Self::with_content(item_size, gap, num_columns, Vec::new())
    }

    pub const fn with_content(item_size: Vector2, gap: Vector2, num_columns: NonZeroU32, content: Vec<T>) -> Self {
        Self {
            layout: UniformGridLayout { item_size, row_gap: gap.y, column_gap: gap.x, num_columns },
            content,
        }
    }

    pub fn from_iter(item_size: Vector2, gap: Vector2, num_columns: NonZeroU32, content: impl IntoIterator<Item = T>) -> Self {
        Self::with_content(item_size, gap, num_columns, Vec::from_iter(content))
    }

    pub fn position(&self, relative_point: Vector2) -> Option<usize> {
        if relative_point.x >= 0.0 && relative_point.y >= 0.0 {
            let slot_width = self.layout.item_size.x + self.layout.column_gap;
            let slot_height = self.layout.item_size.y + self.layout.row_gap;
            let (col, col_region) = (relative_point.x / slot_width, relative_point.x % slot_width);
            if (0.0..u32::MAX as f32).contains(&col) && col_region <= self.layout.item_size.x {
                let col_index = col as u32;
                if col_index < self.layout.num_columns.get() {
                    let (row, row_region) = (relative_point.y / slot_height, relative_point.y % slot_height);
                    if (0.0..u32::MAX as f32).contains(&row) && row_region <= self.layout.item_size.y {
                        let row_index = row as u32;
                        let index = row_index * self.layout.num_columns.get() + col_index;
                        if index < self.content.len() as u32 {
                            return Some(index as usize);
                        }
                    }
                }
            }
        }
        None
    }
}

impl<T: Node> Node for UniformGridNode<T> {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        let (num_rows, num_cols) = (
            self.content.len() as u32 / self.layout.num_columns,
            self.content.len() as u32 % self.layout.num_columns,
        );

        let width  = num_cols as f32 * (self.layout.column_gap + self.layout.item_size.x) - self.layout.column_gap;
        let height = num_rows as f32 * (self.layout.   row_gap + self.layout.item_size.y) - self.layout.   row_gap;

        ((width, Some(width)), (height, Some(height)))
    }

    fn bounds(&self, slot: Rectangle) -> Rectangle {
        let Rectangle { x, y, width: _, height: _ } = slot;
        let ((width, max_width), (height, max_height)) = self.size_range();
        debug_assert_eq!((max_width, max_height), (Some(width), Some(height)),
            "2025-3-24 [`UniformGridNode::bounds()`] implementation written with assumption of non-expandable size");
        Rectangle { x, y, width, height }
    }

    #[inline]
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        for (item, slot) in self.children_mut(slot) {
            item.tick(rl, thread, slot, events);
        }
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        for (item, slot) in self.children(slot) {
            item.draw(d, slot);
        }
    }
}

impl<T: Node> CollectionNode for UniformGridNode<T> {
    type Item = T;
    type Iter<'a> = std::iter::Zip<std::slice::Iter<'a, T>, Iter<'a>> where Self: 'a;
    type IterMut<'a> = std::iter::Zip<std::slice::IterMut<'a, T>, Iter<'a>> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rectangle) -> Self::Iter<'_> {
        self.content.iter().zip(Iter::new(&self.layout, slot))
    }

    #[inline]
    fn children_mut(&mut self, slot: Rectangle) -> Self::IterMut<'_> {
        self.content.iter_mut().zip(Iter::new(&self.layout, slot))
    }
}
