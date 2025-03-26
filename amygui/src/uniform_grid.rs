use std::num::NonZeroU32;
use crate::*;

pub struct Iter<'a, I> {
    iter: I,
    layout: &'a UniformGridLayout,
    front: u32,
    back: u32,
    src_xy: Point,
}

impl<'a, I> Iter<'a, I> {
    const fn new(iter: I, len: u32, layout: &'a UniformGridLayout, slot: Rect) -> Self {
        Self {
            iter,
            layout,
            front: 0,
            back: len,
            src_xy: slot.min_point(),
        }
    }

    fn rect(&self, index: u32) -> Rect {
        let (width, height) = (self.layout.item_width, self.layout.item_height);
        let (row, col) = (
            index / self.layout.num_columns,
            index % self.layout.num_columns,
        );
        let x_min = self.src_xy.x + col as f32 * (width + self.layout.column_gap);
        let y_min = self.src_xy.y + row as f32 * (height + self.layout.row_gap);
        Rect {
            x_min,
            y_min,
            x_max: x_min + width,
            y_max: y_min + height,
        }
    }
}

impl<'a, I: Iterator> Iterator for Iter<'a, I> {
    type Item = (I::Item, Rect);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            let index = self.front;
            self.front += 1;
            Some((item, self.rect(index)))
        } else { None }
    }
}

impl<'a, I: DoubleEndedIterator> DoubleEndedIterator for Iter<'a, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next_back() {
            self.back -= 1;
            let index = self.back;
            Some((item, self.rect(index)))
        } else { None }
    }
}

#[derive(Clone, Copy)]
pub struct UniformGridLayout {
    pub item_width: f32,
    pub item_height: f32,
    pub column_gap: f32,
    pub row_gap: f32,
    pub num_columns: NonZeroU32,
}

pub struct UniformGridNode<T> {
    pub layout: UniformGridLayout,
    pub content: Vec<T>,
}

impl<T> UniformGridNode<T> {
    pub const fn new(item_width: f32, item_height: f32, column_gap: f32, row_gap: f32, num_columns: NonZeroU32) -> Self {
        Self::with_content(item_width, item_height, column_gap, row_gap, num_columns, Vec::new())
    }

    pub const fn with_content(item_width: f32, item_height: f32, column_gap: f32, row_gap: f32, num_columns: NonZeroU32, content: Vec<T>) -> Self {
        Self {
            layout: UniformGridLayout { item_width, item_height, column_gap, row_gap, num_columns },
            content,
        }
    }

    pub fn from_iter(item_width: f32, item_height: f32, column_gap: f32, row_gap: f32, num_columns: NonZeroU32, content: impl IntoIterator<Item = T>) -> Self {
        Self::with_content(item_width, item_height, column_gap, row_gap, num_columns, Vec::from_iter(content))
    }

    pub fn position(&self, relative_point: Point) -> Option<usize> {
        if relative_point.x >= 0.0 && relative_point.y >= 0.0 {
            let slot_width  = self.layout.item_width  + self.layout.column_gap;
            let slot_height = self.layout.item_height + self.layout.row_gap;
            let (col, col_region) = (relative_point.x / slot_width, relative_point.x % slot_width);
            if (0.0..u32::MAX as f32).contains(&col) && col_region <= self.layout.item_width {
                let col_index = col as u32;
                if col_index < self.layout.num_columns.get() {
                    let (row, row_region) = (relative_point.y / slot_height, relative_point.y % slot_height);
                    if (0.0..u32::MAX as f32).contains(&row) && row_region <= self.layout.item_height {
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

        let width  = num_cols as f32 * (self.layout.column_gap + self.layout.item_width ) - self.layout.column_gap;
        let height = num_rows as f32 * (self.layout.   row_gap + self.layout.item_height) - self.layout.   row_gap;

        ((width, Some(width)), (height, Some(height)))
    }
}

impl<T: Node> CollectionNode for UniformGridNode<T> {
    type Item = T;
    type Iter<'a> = Iter<'a, std::slice::Iter<'a, T>> where Self: 'a;
    type IterMut<'a> = Iter<'a, std::slice::IterMut<'a, T>> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        Iter::new(self.content.iter(), self.content.len() as u32, &self.layout, slot)
    }

    #[inline]
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        let n = self.content.len() as u32;
        Iter::new(self.content.iter_mut(), n, &self.layout, slot)
    }
}

impl<T: Node> SimpleCollectionNode for UniformGridNode<T> {}
