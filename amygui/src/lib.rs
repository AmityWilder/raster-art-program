pub mod align_box;
pub mod area_box;
pub mod button;
pub mod events;
pub mod label;
pub mod option;
pub mod overlay_box;
pub mod pad_box;
pub mod region;
pub mod size_box;
pub mod split_box;
pub mod stack_box;
pub mod uniform_grid;

pub(crate) use events::Events;

pub mod prelude {
    pub use crate::{
        padding,
        impl_guinode_union,
        InputBackend,
        TickBackend,
        DrawBackend,
        Point,
        Rect,
        Visibility,
        Direction,
        Node,
        TickNode,
        DrawNode,
        ParentNode,
        SimpleParentNode,
        CollectionNode,
        SimpleCollectionNode,
        GUINode,
        AmyGUINode,
        Empty,
        align_box::{
            Align,
            AlignBoxLayout,
            AlignBoxNode,
        },
        area_box::{
            AreaBoxLayout,
            AreaBoxNode,
        },
        button::{
            ButtonStyle,
            ButtonState,
            Button,
        },
        events::{
            Event,
            MouseEvent,
            Events,
        },
        label::{
            LabelStyle,
            Label,
        },
        overlay_box::OverlayBox,
        pad_box::{
            PadBoxLayout,
            PadBoxNode,
        },
        region::Region,
        size_box::{
            SizeBoxLayout,
            SizeBoxNode,
        },
        split_box::{
            SplitBoxLayout,
            SplitBoxNode,
        },
        stack_box::{
            StackBoxLayout,
            StackBoxNode,
        },
        uniform_grid::{
            UniformGridLayout,
            UniformGridNode,
        },
    };
}
use prelude::*;

pub trait InputBackend {
    fn mouse_position(&mut self) -> Point;
    fn is_m1_pressed(&mut self) -> bool;
    fn is_m1_released(&mut self) -> bool;
    fn mouse_wheel_move(&mut self) -> Point;
}

pub trait TickBackend {}

pub trait DrawBackend {
    type Color: Copy;

    fn draw_rect(&mut self, rect: &Rect, color: &Self::Color);
    fn draw_text(&mut self, text: &str, top_left: Point, font_size: f32, color: &Self::Color);
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
    fn bounds(&self, slot: Rect) -> Rect {
        let ((min_width, max_width), (min_height, max_height)) = self.size_range();
        let Rect { x_min, y_min, x_max, y_max } = slot;
        let (width, height) = (x_max - x_min, y_max - y_min);
        debug_assert!(min_width <= width && min_height <= height, "slot should not be smaller than minimum size");
        let x_max = x_min + max_width .map_or(width,  |w| width .min(w));
        let y_max = x_min + max_height.map_or(height, |h| height.min(h));
        Rect { x_min, y_min, x_max, y_max }
    }
}

/// **Note:** Always tick children BEFORE self (if they are going to tick at all).
/// The foremost nodes should have priority for event occlusion.
pub trait TickNode<TB>: Node {
    /// Handle reserving events for objects that are currently in a state that gives them exclusive rights to those events.
    /// Example: A viewport was clicked, and now it has priviledged access to hover events even if the mouse exits the
    /// viewport or hovers something else.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    #[allow(unused)]
    fn dibs_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {}

    /// Tick that occurs only while hovered.
    ///
    /// Nodes with children should always call this recursively,
    /// and call `inactive_tick` on children that are not hovered.
    ///
    /// Default implementation calls inactive tick.
    #[inline]
    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        self.inactive_tick(tb, slot, events);
    }

    /// Tick that occurs when the element is not hovered.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    #[allow(unused)]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) {}
}

/// **Note:** Always draw children AFTER self (if they are going to draw at all).
/// The foremost nodes should always be drawn after everything else, since that
/// will display them in front.
pub trait DrawNode<DB>: Node {
    /// Draw the node.
    ///
    /// Nodes with children should always call this recursively.
    #[inline]
    #[allow(unused)]
    fn draw(&self, d: &mut DB, slot: Rect) {}
}

pub trait ParentNode: Node {
    type Item: Node;

    fn content(&self) -> &Self::Item;
    fn content_mut(&mut self) -> &mut Self::Item;

    #[inline]
    fn child(&self, slot: Rect) -> (&Self::Item, Rect) {
        (self.content(), self.bounds(slot))
    }

    #[inline]
    fn child_mut(&mut self, mut slot: Rect) -> (&mut Self::Item, Rect) {
        slot = self.bounds(slot);
        (self.content_mut(), slot)
    }
}

/// Alias for [`SimpleCollectionNode`]
pub trait SimpleParentNode: ParentNode {}

impl<T: SimpleParentNode> SimpleCollectionNode for T {}

impl<T: ParentNode> CollectionNode for T {
    type Item = T::Item;
    type Iter<'a> = std::iter::Once<(&'a Self::Item, Rect)> where Self: 'a, Self::Item: 'a;
    type IterMut<'a> = std::iter::Once<(&'a mut Self::Item, Rect)> where Self: 'a, Self::Item: 'a;

    #[inline]
    fn children(&self, slot: Rect) -> Self::Iter<'_> {
        std::iter::once(self.child(slot))
    }

    #[inline]
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_> {
        std::iter::once(self.child_mut(slot))
    }
}

pub trait CollectionNode: Node {
    type Item: Node;
    type Iter<'a>: DoubleEndedIterator<Item = (&'a Self::Item, Rect)> where Self: 'a, Self::Item: 'a;
    type IterMut<'a>: DoubleEndedIterator<Item = (&'a mut Self::Item, Rect)> where Self: 'a, Self::Item: 'a;

    fn children(&self, slot: Rect) -> Self::Iter<'_>;
    fn children_mut(&mut self, slot: Rect) -> Self::IterMut<'_>;
}

/// Automatically provides default collection implementations of [`TickNode`] and [`DrawNode`]
/// that iterate over each slot and call recursively.
///
/// Don't use this if your node needs a custom tick/draw implementation.
pub trait SimpleCollectionNode: CollectionNode {}

impl<TB, T: SimpleCollectionNode<Item: TickNode<TB>>> TickNode<TB> for T {
    #[inline]
    fn dibs_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        for (item, slot) in self.children_mut(slot).rev() {
            item.dibs_tick(tb, slot, events);
        }
    }

    #[inline]
    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        for (item, slot) in self.children_mut(slot).rev() {
            if events.hover.is_some_and_overlapping(slot) {
                item.active_tick(tb, slot, events);
            } else {
                item.inactive_tick(tb, slot, events);
            }
        }
    }

    #[inline]
    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) {
        for (item, slot) in self.children_mut(slot).rev() {
            item.inactive_tick(tb, slot, events);
        }
    }
}

impl<DB, T: SimpleCollectionNode<Item: DrawNode<DB>>> DrawNode<DB> for T {
    #[inline]
    fn draw(&self, d: &mut DB, slot: Rect) {
        for (item, slot) in self.children(slot) {
            item.draw(d, slot);
        }
    }
}

pub trait GUINode<TB, DB>: Node + TickNode<TB> + DrawNode<DB> {}

impl<TB, DB, T: Node + TickNode<TB> + DrawNode<DB>> GUINode<TB, DB> for T {}

pub struct Empty;

impl Node for Empty {}
impl<TB> TickNode<TB> for Empty {}
impl<DB> DrawNode<DB> for Empty {}

#[macro_export]
macro_rules! impl_guinode_union {
    (

        $(#[$meta:meta])*
        $vis:vis enum$(($($EnumGen:tt)*))? $Enum:ident$(<$($Gen:ident),* $(,)?>)? {
            $($Variant:ident($Node:ty)),* $(,)?
        }
        impl$(($($NodeGen:tt)*))? Node;
        impl$(($($TickGen:tt)*))? Tick<($($TB:tt)+)>;
        impl$(($($DrawGen:tt)*))? Draw<($($DB:tt)+)>;
    ) => {
        $(#[$meta])*
        $vis enum $Enum$(<$($EnumGen)*>)? {
            $($Variant($Node)),*
        }
        impl$(<$($NodeGen)*>)? Node for $Enum$(<$($Gen),*>)? {
            #[inline]
            fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
                match self {
                    $(Self::$Variant(node) => node.size_range(),)*
                }
            }
            #[inline]
            fn bounds(&self, slot: Rect) -> Rect {
                match self {
                    $(Self::$Variant(node) => node.bounds(slot),)*
                }
            }
        }
        impl$(<$($TickGen)*>)? TickNode<$($TB)+> for $Enum$(<$($Gen),*>)? {
            #[inline]
            fn dibs_tick(&mut self, tb: &mut $($TB)+, slot: Rect, events: &mut Events) {
                match self {
                    $(Self::$Variant(node) => node.dibs_tick(tb, slot, events),)*
                }
            }
            #[inline]
            fn active_tick(&mut self, tb: &mut $($TB)+, slot: Rect, events: &mut Events) {
                match self {
                    $(Self::$Variant(node) => node.active_tick(tb, slot, events),)*
                }
            }
            #[inline]
            fn inactive_tick(&mut self, tb: &mut $($TB)+, slot: Rect, events: &Events) {
                match self {
                    $(Self::$Variant(node) => node.inactive_tick(tb, slot, events),)*
                }
            }
        }
        impl$(<$($DrawGen)*>)? DrawNode<$($DB)+> for $Enum$(<$($Gen),*>)? {
            #[inline]
            fn draw(&self, d: &mut $($DB)+, slot: Rect) {
                match self {
                    $(Self::$Variant(node) => node.draw(d, slot),)*
                }
            }
        }
    };
}

impl_guinode_union!{
    /// A union of all AmityGUI nodes.
    pub enum(ColorT: Copy, T) AmyGUINode<ColorT, T> {
        AlignBox(AlignBoxNode<T>),
        AreaBox(AreaBoxNode<T>),
        Button(Button<ColorT, T>),
        Label(Label<ColorT>),
        PadBox(PadBoxNode<T>),
        SizeBox(SizeBoxNode<T>),
        SplitBox(SplitBoxNode<T>),
        StackBox(StackBoxNode<T>),
        UniformGrid(UniformGridNode<T>),
        Empty(Empty),
    }
    impl(ColorT: Copy, T: Node) Node;
    impl(ColorT: Copy, TB, T: TickNode<TB>) Tick<(TB)>;
    impl(ColorT: Copy, DB: DrawBackend<Color = ColorT>, T: DrawNode<DB>) Draw<(DB)>;
}
