pub mod size_box;
pub mod split_box;
pub mod pad_box;
pub mod stack_box;
pub mod uniform_grid;
pub mod button;
pub mod region;
pub mod option;
pub mod align_box;
pub mod events;
pub mod label;
pub mod area_box;

pub(crate) use events::Events;

pub trait InputBackend {
    fn mouse_position(&mut self) -> Point;
    fn is_m1_pressed(&mut self) -> bool;
    fn is_m1_released(&mut self) -> bool;
    fn mouse_wheel_move(&mut self) -> Point;
}

pub trait TickBackend {}

pub trait DrawBackend {
    type Color: Copy;

    fn draw_rect(&mut self, rect: &Rect, color: Self::Color);
    fn draw_text(&mut self, text: &str, top_left: Point, font_size: f32, color: Self::Color);
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl Rect {
    #[inline]
    pub const fn contains(&self, point: Point) -> bool {
        self.x_min <= point.x && point.x < self.x_max &&
        self.y_min <= point.y && point.y < self.y_max
    }

    #[inline]
    pub const fn min_point(&self) -> Point {
        Point { x: self.x_min, y: self.y_min }
    }

    #[inline]
    pub const fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    #[inline]
    pub const fn height(&self) -> f32 {
        self.y_max - self.y_min
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Occlude hit test from parent nodes
    Occlude,
    /// Skip hit test this node, still perform test on children
    PassthroughSelf,
    /// Skip hit test for children, still perform test on self
    PassthroughChildren,
    /// Skip hit test for self and children, still appear visible
    Passthrough,
    /// Hide self and children from both hit test and rendering but still take up space
    Phantom,
    /// Hide self and children from both hit test and rendering and take up no space
    Collapsed,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Row,
    Column,
}

pub trait Node<TB, DB> {
    /// Give the minmum and maximum for the node's width and height.
    /// [`None`] represents unbounded maximum size, and is determined by what can fit inside the slot.
    /// Minimum size should always be at least 0.
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        ((0.0, None), (0.0, None))
    }

    /// The rectangle that fully contains the "interactable" region of the node.
    /// Defaults to the slot rectangle shrank to the max size (top left corner).
    #[inline]
    fn bounds(&self, slot: Rect) -> Rect {
        let ((min_width, max_width), (min_height, max_height)) = self.size_range();
        let Rect { x_min, y_min, x_max, y_max } = slot;
        let (width, height) = (x_max - x_min, y_max - y_min);
        debug_assert!(min_width <= width && min_height <= height, "slot should not be smaller than minimum size");
        let x_max = x_min + max_width .map_or(width,  |w| width .min(w));
        let y_max = x_min + max_height.map_or(height, |h| height.min(h));
        Rect { x_min, y_min, x_max, y_max }
    }

    /// Handle reserving events for objects that are currently in a state that gives them exclusive rights to those events.
    /// Example: A viewport was clicked, and now it has priviledged access to hover events even if the mouse exits the
    /// viewport or hovers something else.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    #[allow(unused)]
    fn dibs_tick(&mut self, slot: Rect, events: &mut Events) {}

    /// Tick that occurs only while hovered.
    ///
    /// Nodes with children should always call this recursively,
    /// and call `inactive_tick` on children that are not hovered.
    ///
    /// Default implementation calls inactive tick.
    #[inline]
    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) where TB: TickBackend {
        self.inactive_tick(tb, slot, events);
    }

    /// Tick that occurs when the element is not hovered.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    #[allow(unused)]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) where TB: TickBackend {}

    /// Draw the node.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    #[allow(unused)]
    fn draw(&self, d: &mut DB, slot: Rect) where DB: DrawBackend {}
}

pub trait ParentNode<TB, DB> {
    type Item: Node<TB, DB>;

    fn child(&self, slot: Rect) -> (&Self::Item, Rect);
    fn child_mut(&mut self, slot: Rect) -> (&mut Self::Item, Rect);
}

pub trait CollectionNode<TB, DB> {
    type Item: Node<TB, DB>;
    type Iter<'a>: Iterator<Item = (&'a Self::Item, Rect)> where Self: 'a, Self::Item: 'a;
    type IterMut<'a>: Iterator<Item = (&'a mut Self::Item, Rect)> where Self: 'a, Self::Item: 'a;

    fn children(&self, slot: Rect) -> Self::Iter<'_>;
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_>;
}

pub struct Empty;

impl<TB: TickBackend, DB: DrawBackend> Node<TB, DB> for Empty {}

impl<TB: TickBackend, DB: DrawBackend> Node<TB, DB> for Box<dyn Node<TB, DB>> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.as_ref().size_range()
    }

    #[inline]
    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        self.as_mut().active_tick(tb, slot, events);
    }

    #[inline]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) {
        self.as_mut().inactive_tick(tb, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        self.as_ref().draw(d, slot);
    }
}
