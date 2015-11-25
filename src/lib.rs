#![allow(unused)]
extern crate clock_ticks;

mod svg;

use std::cell::{RefCell, Cell};
use std::iter::Peekable;
use std::borrow::Cow;

pub type StrCow = Cow<'static, str>;

thread_local!(static LIBRARY: RefCell<Library> = RefCell::new(Library::new()));

#[derive(Debug)]
struct Library {
    past: Vec<PrivateFrame>,
    current: Option<PrivateFrame>
}

#[derive(Debug)]
struct PrivateFrame {
    next_id: u32,
    all: Vec<Event>,
    id_stack: Vec<u32>,
}

#[derive(Debug)]
struct Event {
    id: u32,
    parent: Option<u32>,
    name: StrCow,
    start_ns: u64,
    end_ns: Option<u64>,
    delta: Option<u64>,
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
    pub start_ns: u64,
    /// The timestamp of the end of the span
    pub end_ns: u64,
    /// The time that ellapsed between start_ns and end_ns
    pub delta: u64,
    /// How deep this span is in the tree
    pub depth: u16,
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

pub struct SpanGuard {
    name: Option<StrCow>
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let name = self.name.take().unwrap();
        end(name);
    }
}

impl SpanGuard {
    pub fn end(self) { }
}

fn convert_events_to_span<'a, I>(events: I) -> Vec<Span>
where I: Iterator<Item = &'a Event> {
    let mut iterator = events.peekable();
    let mut v = vec![];
    while let Some(event) = iterator.next() {
        if let Some(span) = event_to_span(event, &mut iterator, 0) {
            v.push(span);
        }
    }
    v
}

fn event_to_span<'a, I: Iterator<Item = &'a Event>>(event: &Event, events: &mut Peekable<I>, depth: u16) -> Option<Span> {
    if event.end_ns.is_some() && event.delta.is_some() {
        let mut span = Span {
            name: event.name.clone(),
            start_ns: event.start_ns,
            end_ns: event.end_ns.unwrap(),
            delta: event.delta.unwrap(),
            depth: depth,
            children: vec![],
            notes: event.notes.clone(),
            _priv: ()
        };

        loop {
            {
                match events.peek() {
                    Some(next) if next.parent != Some(event.id) => break,
                    None => break,
                    _ => {}
                }
            }

            let next = events.next().unwrap();
            let child = event_to_span(next, events, depth + 1);
            if let Some(child) = child {
                span.children.push(child);
            }
        }
        Some(span)
    } else {
        None
    }
}


impl Frame {
    fn from_private(p: &PrivateFrame) -> Frame {
        let v = convert_events_to_span(p.all.iter());
        Frame { roots: v, _priv: () }
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

/// Starts a Span and also returns a SpanGuard.
///
/// When the SpanGuard is dropped (or the .end() is called on it),
/// the span will automatically be ended.
pub fn start_guard<S: Into<StrCow>>(name: S) -> SpanGuard {
    let name = name.into();
    start(name.clone());
    SpanGuard { name: Some(name) }
}

/// Starts and ends a Span that lasts for the duration of the
/// function `f`.
pub fn span_of<S, F, R>(name: S, f: F) -> R where
S: Into<StrCow>,
F: FnOnce() -> R
{
    let name = name.into();
    start(name.clone());
    let r = f();
    end(name);
    r
}

/// Starts a new Span
pub fn start<S: Into<StrCow>>(name: S) {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        if library.current.is_none() {
            library.current = Some(PrivateFrame {
                next_id: 0,
                all: vec![],
                id_stack: vec![],
            });
        }

        let collector = library.current.as_mut().unwrap();
        let id = collector.next_id;
        collector.next_id += 1;

        let this = Event {
            id: id,
            parent: collector.id_stack.last().cloned(),
            name: name.into(),
            start_ns: clock_ticks::precise_time_ns(),
            end_ns: None,
            delta: None,
            notes: vec![]
        };

        collector.all.push(this);
        collector.id_stack.push(id);
    });
}

/// Ends the current Span and returns the number
/// of nanoseconds that passed.
pub fn end<S: Into<StrCow>>(name: S) -> u64 {
    let name = name.into();
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        let collector = match library.current.as_mut() {
            Some(x) => x,
            None => panic!("flame::end({}) called without a currently running span!", &name)
        };

        let current_id = match collector.id_stack.pop() {
            Some(id) => id,
            None => panic!("flame::end({:?}) called without a currently running span!",
                          &name)
        };
        let event = &mut collector.all[current_id as usize];

        if event.name != name {
            panic!("flame::end({}) attempted to end {}", &name, event.name);
        } else {
            let timestamp = clock_ticks::precise_time_ns();
            event.end_ns = Some(timestamp);
            event.delta = Some(timestamp - event.start_ns);
            event.delta
        }
    }).unwrap()
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

        let current_id = match collector.id_stack.last() {
            Some(id) => *id,
            None => panic!("flame::note({}, {:?}) called without a currently running span!",
                           &name, &description)
        };

        let event = &mut collector.all[current_id as usize];
        event.notes.push(Note {
            name: name,
            description: description,
            instant: clock_ticks::precise_time_ns(),
            _priv: ()
        });
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
        println!("{:?}", library);
    });
}

pub use svg::dump_svg;
