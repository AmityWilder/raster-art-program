use crate::*;

struct PanZoomEvents<'a> {
    original_mouse_pos: Option<Point>,
    events: &'a mut Events,
}

impl Drop for PanZoomEvents<'_> {
    fn drop(&mut self) {
        if let Some(event) = self.events.mouse_event.event.as_mut() {
            let original = self.original_mouse_pos.expect("original position should be Some if mouse event exists");
            event.position.x = original.x;
            event.position.y = original.y;
        }
    }
}

impl<'a> PanZoomEvents<'a> {
    pub fn new(events: &'a mut Events, pan: Point, zoom: f32) -> Self {
        let original_mouse_pos;
        if let Some(event) = events.mouse_event.event.as_mut() {
            original_mouse_pos = Some(event.position);
            event.position.x = (event.position.x + pan.x) * zoom;
            event.position.y = (event.position.y + pan.y) * zoom;
        } else {
            original_mouse_pos = None;
        }
        Self {
            original_mouse_pos,
            events,
        }
    }
}

/// A UI element whose contents can be zoomed and panned
pub struct Viewport<T> {
    pub pan: Point,
    pub zoom: f32,
    pub width: f32,
    pub height: f32,
    pub content: T,
}

impl<T: Node> Node for Viewport<T> {
    #[inline]
    fn size_range(&self) -> ((f32, Option<f32>), (f32, Option<f32>)) {
        ((self.width, Some(self.width)), (self.height, Some(self.height)))
    }
}

impl<TB, T: TickNode<TB>> TickNode<TB> for Viewport<T> {
    fn dibs_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        let vp_events = PanZoomEvents::new(events, self.pan, self.zoom);
        let slot = self.bounds(slot);
        self.content.dibs_tick(tb, slot, vp_events.events);
    }

    fn active_tick(&mut self, tb: &mut TB, slot: Rect, events: &mut Events) {
        let vp_events = PanZoomEvents::new(events, self.pan, self.zoom);
        let slot = self.bounds(slot);
        self.content.active_tick(tb, slot, vp_events.events);
    }

    fn inactive_tick(&mut self, tb: &mut TB, slot: Rect, events: &Events) {
        let mut events = *events;
        let vp_events = PanZoomEvents::new(&mut events, self.pan, self.zoom);
        let slot = self.bounds(slot);
        self.content.inactive_tick(tb, slot, vp_events.events);
    }
}

impl<DB, T: DrawNode<DB>> DrawNode<DB> for Viewport<T> {
    fn draw(&self, d: &mut DB, slot: Rect) {

    }
}

impl<T: Node> ParentNode for Viewport<SizeBoxNode<T>> {
    type Item = SizeBoxNode<T>;

    #[inline]
    fn content(&self) -> &Self::Item {
        &self.content
    }

    #[inline]
    fn content_mut(&mut self) -> &mut Self::Item {
        &mut self.content
    }
}