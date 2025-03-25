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
pub(crate) use raylib::prelude::*;

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

pub trait Node {
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
    fn bounds(&self, slot: Rectangle) -> Rectangle {
        let Rectangle { x, y, width, height } = slot;
        let ((_, max_width), (_, max_height)) = self.size_range();
        let width = max_width.map_or(width, |w| width.min(w));
        let height = max_height.map_or(height, |h| height.min(h));
        Rectangle { x, y, width, height }
    }

    /// Handle reserving events for objects that are currently in a state that gives them exclusive rights to those events.
    /// Example: A viewport was clicked, and now it has priviledged access to hover events even if the mouse exits the
    /// viewport or hovers something else.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    fn dibs_tick(&mut self, slot: Rectangle, events: &mut Events) {}

    /// Tick that occurs only while hovered.
    ///
    /// Nodes with children should always call this recursively,
    /// and call `inactive_tick` on children that are not hovered.
    ///
    /// Default implementation calls inactive tick.
    #[inline]
    fn active_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        self.inactive_tick(rl, thread, slot, events);
    }

    /// Tick that occurs when the element is not hovered.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    fn inactive_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &Events) {}

    /// Draw the node.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {}
}

pub trait ParentNode {
    type Item: Node;

    fn child(&self, slot: Rectangle) -> (&Self::Item, Rectangle);
    fn child_mut(&mut self, slot: Rectangle) -> (&mut Self::Item, Rectangle);
}

pub trait CollectionNode {
    type Item: Node;
    type Iter<'a>: Iterator<Item = (&'a Self::Item, Rectangle)> where Self: 'a;
    type IterMut<'a>: Iterator<Item = (&'a mut Self::Item, Rectangle)> where Self: 'a;

    fn children(&self, slot: Rectangle) -> Self::Iter<'_>;
    fn children_mut(&mut self, slot: Rectangle) -> Self::IterMut<'_>;
}

impl<T: ParentNode> CollectionNode for T {
    type Item = T::Item;
    type Iter<'a> = std::iter::Once<(&'a Self::Item, Rectangle)> where Self: 'a;
    type IterMut<'a> = std::iter::Once<(&'a mut Self::Item, Rectangle)> where Self: 'a;

    #[inline]
    fn children(&self, slot: Rectangle) -> Self::Iter<'_> {
        std::iter::once(self.child(slot))
    }

    #[inline]
    fn children_mut(&mut self, slot: Rectangle) -> Self::IterMut<'_> {
        std::iter::once(self.child_mut(slot))
    }
}

pub struct Empty;

impl Node for Empty {}

impl Node for Box<dyn Node> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.as_ref().size_range()
    }

    #[inline]
    fn active_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        self.as_mut().active_tick(rl, thread, slot, events);
    }

    #[inline]
    fn inactive_tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &Events) {
        self.as_mut().inactive_tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        self.as_ref().draw(d, slot);
    }
}
