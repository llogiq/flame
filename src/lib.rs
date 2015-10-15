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
    past: Vec<PrivateFrame>,
    current: Option<PrivateFrame>
}

#[derive(Debug)]
struct PrivateFrame {
    root: RcEvent,
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
    notes: Vec<Note>,
}

/// A chunk of spans that are meant to be grouped together into a "frame".
///
/// The naming (and a possible usecase) comes from gaming, where a bunch
/// of logic and rendering happens repeatedly every "frame".
/// When developing a game, a flamegraph can be used to analyse and debug
/// performance issues when you see that a particular frame is oddly shaped.
///
/// If you don't have any sort of repeatable logic that you'd like to
/// show off in your flamegraph, using a single frame is totally acceptable.
#[derive(Debug)]
pub struct Frame {
    /// A list of spans contained inside this frame.
    pub roots: Vec<Span>,
    _priv: (),
}

/// A named timespan.
///
/// The span is the most important feature of Flame.  It denotes
/// a chunk of time that is important to you.
///
/// The Span records
/// * Start and stop time
/// * A list of children (also called sub-spans)
/// * A list of notes
#[derive(Debug)]
pub struct Span {
    /// The name of the span
    pub name: StrCow,
    /// The timestamp of the start of the span
    pub start_ms: u64,
    /// The timestamp of the end of the span
    pub end_ms: u64,
    /// The time that ellapsed between start_ms and end_ms
    pub delta: u64,
    /// A list of spans that occurred inside this one
    pub children: Vec<Span>,
    /// A list of notes that occurred inside this span
    pub notes: Vec<Note>,
    _priv: (),
}

/// A note for use in debugging.
#[derive(Debug, Clone)]
pub struct Note {
    /// A short name describing what happened at some instant in time
    pub name: StrCow,
    /// A longer description
    pub description: Option<StrCow>,
    /// The time that the note was added
    pub instant: u64,
    _priv: (),
}

impl Span {
    fn from_private(p: &Event, into: &mut Vec<Span>) {
        if p.end_ms.is_some() && p.delta.is_some() {
            let mut public = Span {
                name: p.name.clone(),
                start_ms: p.start_ms,
                end_ms: p.end_ms.unwrap(),
                delta: p.delta.unwrap(),
                children: Vec::new(),
                notes: p.notes.clone(),
                _priv: (),
            };

            for child in p.children.iter() {
                Span::from_private(&child.borrow(), &mut public.children);
            }

            into.push(public);
        }
    }
}

impl Frame {
    fn from_private(p: &PrivateFrame) -> Frame {
        let root = p.root.borrow();
        let mut v = Vec::with_capacity(root.children.len());
        for child in root.children.iter() {
            Span::from_private(&child.borrow(), &mut v);
        }
        Frame { roots: v, _priv: () }
    }
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

/// Starts a new Span
pub fn start<S: Into<StrCow>>(name: S) {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            let root = Event::root();
            library.current = Some(PrivateFrame {
                root: root.clone(),
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

/// Ends the current Span
pub fn end<S: Into<StrCow>>(name: S) {
    let name = name.into();
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            panic!("flame::event_end({}) called without a currently running span!", &name);
        }

        let collector = library.current.as_mut().unwrap();

        if collector.current.is_none() {
            panic!("flame::event_end({}) called without a currently running span!", &name);
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

/// Records a note on the current Span.
pub fn note<S: Into<StrCow>>(name: S, description: Option<S>) {
    let name = name.into();
    let description = description.map(Into::into);

    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            panic!("flame::note({:?}) called without a currently running span!", &name);
        }

        let collector = library.current.as_mut().unwrap();

        if collector.current.is_none() {
            panic!("flame::note({:?}) called without a currently running span!", &name)
        }

        let current = collector.current.as_mut().unwrap();
        let mut current = current.borrow_mut();

        let note = Note {
            name: name,
            description: description,
            instant: clock_ticks::precise_time_ms(),
            _priv: ()
        };

        current.notes.push(note);
    });
}

/// Starts a new frame.
pub fn next_frame() {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if let Some(prev) = library.current.take() {
            library.past.push(prev);
        }
    });
}

/// Clears all of the recorded info that Flame has
/// tracked.
pub fn clear() {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        library.past = vec![];
        library.current = None;
    });
}

/// Returns a list of frames
pub fn frames() -> Vec<Frame> {
    let mut out = vec![];
    LIBRARY.with(|library| {
        let library = library.borrow();
        for past in library.past.iter() {
            out.push(Frame::from_private(past));
        }
        if let Some(cur) = library.current.as_ref() {
            out.push(Frame::from_private(cur))
        }
    });
    out
}

/// Prints all of the frames to stdout.
pub fn debug() {
    LIBRARY.with(|library| {
        println!("{:?}", frames());
    });
}
