pub mod size_box;
pub mod split_box;
pub mod pad_box;
pub mod stack_box;
pub mod uniform_grid;
pub mod button;
pub mod region;
pub mod option;
pub mod align_box;

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

pub struct Events {
    pub hover: Option<Vector2>,
    pub left_mouse_press: Option<()>,
    pub scroll: Option<Vector2>,
}

impl Events {
    pub fn check(rl: &RaylibHandle) -> Self {
        Self {
            hover: Some(rl.get_mouse_position()),
            left_mouse_press: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT).then_some(()),
            scroll: Some(rl.get_mouse_wheel_move_v().into()),
        }
    }
}

pub trait Node {
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>));

    /// The rectangle that fully contains the "interactable" region of the node.
    /// Defaults to the slot rectangle, assuming the box doesn't change the interactable region.
    #[inline]
    fn bounds(&self, slot: Rectangle) -> Rectangle {
        slot
    }

    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events);
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle);
}

pub trait ParentNode: Node {
    type Item: Node;

    fn child(&self, slot: Rectangle) -> (&Self::Item, Rectangle);
    fn child_mut(&mut self, slot: Rectangle) -> (&mut Self::Item, Rectangle);
}

pub trait CollectionNode: Node {
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

pub struct Fill;

impl Node for Fill {
    #[inline(always)]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        ((0.0, None), (0.0, None))
    }

    #[inline(always)]
    fn tick(&mut self, _rl: &mut RaylibHandle, _thread: &RaylibThread, _slot: Rectangle, _events: &mut Events) {}

    #[inline(always)]
    fn draw(&self, _d: &mut RaylibDrawHandle, _slot: Rectangle) {}
}

impl Node for Box<dyn Node> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        self.as_ref().size_range()
    }

    #[inline]
    fn tick(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, slot: Rectangle, events: &mut Events) {
        self.as_mut().tick(rl, thread, slot, events);
    }

    #[inline]
    fn draw(&self, d: &mut RaylibDrawHandle, slot: Rectangle) {
        self.as_ref().draw(d, slot);
    }
}
