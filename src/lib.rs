use std::cell::{RefCell, Cell};
use std::rc::Rc;

thread_local!(static event_collector: EventCollector = RefCell::new(1));

struct EventCollector {
    root: Rc<Event>,
    current: Rc<Event>
}

struct Event {
    name: &'static str,
    start_ms: u32,
    end_ms: Cell<Option<u32>>,
    children: Vec<Event>
}
