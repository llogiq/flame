#![allow(unused)]

extern crate clock_ticks;

use std::cell::{RefCell, Cell};
use std::rc::Rc;
use std::borrow::Cow;

pub type StrCow = Cow<'static, str>;

type RcEvent = Rc<RefCell<Event>>;

thread_local!(static LIBRARY: RefCell<Library> = RefCell::new(Library::new()));

#[derive(Debug)]
struct Library {
    past: Vec<Frame>,
    current: Option<Frame>
}

#[derive(Debug)]
struct Frame {
    root: Option<RcEvent>,
    stack: Vec<RcEvent>,
    current: Option<RcEvent>
}

#[derive(Debug)]
struct Event {
    name: StrCow,
    start_ms: u64,
    end_ms: Option<u64>,
    delta: Option<u64>,
    children: Vec<RcEvent>,
    notes: Vec<StrCow>,
}

impl Event {
    fn root() -> RcEvent {
        Rc::new(RefCell::new(Event {
            name: "<root>".into(),
            start_ms: clock_ticks::precise_time_ms(),
            end_ms: None,
            delta: None,
            children: vec![],
            notes: vec![],
        }))
    }
}


impl Library {
    fn new() -> Library {
        Library {
            past: vec![],
            current: None
        }
    }
}

pub fn start<S: Into<StrCow>>(name: S) {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            let root = Event::root();
            library.current = Some(Frame {
                root: Some(root.clone()),
                stack: vec![],
                current: Some(root)
            });
        }

        let collector = library.current.as_mut().unwrap();

        if let Some(mut prev) = collector.current.take() {
            collector.stack.push(prev);
        }

        collector.current = Some(Rc::new(RefCell::new(Event {
            name: name.into(),
            start_ms: clock_ticks::precise_time_ms(),
            end_ms: None,
            delta: None,
            children: vec![],
            notes: vec![]
        })));

        if let Some(parent) = collector.stack.last_mut() {
            let mut parent = parent.borrow_mut();
            parent.children.push(collector.current.clone().unwrap())
        }
    });
}

pub fn end<S: Into<StrCow>>(name: S) {
    let name = name.into();
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            panic!("flame::event_end({}) called without a currently running event!", &name);
        }

        let collector = library.current.as_mut().unwrap();

        if collector.current.is_none() {
            panic!("flame::event_end({}) called without a currently running event!", &name);
        }

        let current = collector.current.take().unwrap();
        let mut current = current.borrow_mut();

        if current.name == name {
            let end_ms = clock_ticks::precise_time_ms();
            current.end_ms = Some(end_ms);
            current.delta = Some(end_ms - current.start_ms);
            collector.current = collector.stack.pop();
        } else {
            panic!("flame::event_end({}) tried to end the event {}", &name, &current.name);
        }
    });
}

pub fn note<S: Into<StrCow>>(note: S) {
    let note = note.into();
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            panic!("flame::note({}) called without a currently running event!", &note);
        }

        let collector = library.current.as_mut().unwrap();
        
        if collector.current.is_none() {
            panic!("flame::note({}) called without a currently running event!", &note)
        }

        let current = collector.current.as_mut().unwrap();
        let mut current = current.borrow_mut();
        current.notes.push(note);
    });
}

pub fn next_frame() {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if let Some(prev) = library.current.take() {
            library.past.push(prev);
        }
    });
}

pub fn clear() {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        library.past = vec![];
        library.current = None;
    });
}

pub fn debug() {
    LIBRARY.with(|library| {
        println!("{:?}", library);
    });
}
