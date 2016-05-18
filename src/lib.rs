#![allow(unused)]
extern crate clock_ticks;

mod svg;
mod html;

use std::cell::{RefCell, Cell};
use std::iter::Peekable;
use std::borrow::Cow;

pub type StrCow = Cow<'static, str>;

thread_local!(static LIBRARY: RefCell<Library> = RefCell::new(Library::new()));

#[derive(Debug, Default)]
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
    collapse: bool,
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
    collapsable: bool,
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
    name: Option<StrCow>,
    collapse: bool,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let name = self.name.take().unwrap();
        end_impl(name, self.collapse);
    }
}

impl SpanGuard {
    pub fn end(self) { }
    pub fn end_collapse(mut self) {
        self.collapse = true;
    }
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
            collapsable: event.collapse,
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
                // Try to collapse with the previous span
                if span.children.len() != 0 && child.collapsable && child.children.len() == 0 {
                    let last = span.children.last_mut().unwrap();
                    if last.name == child.name && last.depth == child.depth {
                        last.end_ns = child.end_ns;
                        last.delta += child.delta;
                        continue;
                    }
                }
                
                // Otherwise, it's a new node
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

/// Starts a `Span` and also returns a `SpanGuard`.
///
/// When the `SpanGuard` is dropped (or `.end()` is called on it),
/// the span will automatically be ended.
pub fn start_guard<S: Into<StrCow>>(name: S) -> SpanGuard {
    let name = name.into();
    start(name.clone());
    SpanGuard { name: Some(name), collapse: false }
}

/// Starts and ends a `Span` that lasts for the duration of the
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
            collapse: false,
            start_ns: clock_ticks::precise_time_ns(),
            end_ns: None,
            delta: None,
            notes: vec![]
        };

        collector.all.push(this);
        collector.id_stack.push(id);
    });
}

fn end_impl<S: Into<StrCow>>(name: S, collapse: bool) -> u64 {
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
        }

        let timestamp = clock_ticks::precise_time_ns();
        event.end_ns = Some(timestamp);
        event.collapse = collapse;
        event.delta = Some(timestamp - event.start_ns);
        event.delta
    }).unwrap()
}

/// Ends the current Span and returns the number
/// of nanoseconds that passed.
pub fn end<S: Into<StrCow>>(name: S) -> u64 {
    end_impl(name, false)
}

/// Ends the current Span and returns the number of
/// nanoseconds that passed.
///
/// If this span is a leaf node, and the previous span
/// has the same name and depth, then collapse this
/// span into the previous one.  The end_ns field will
/// be updated to the end time of *this* span, and the
/// delta field will be the sum of the deltas from this
/// and the previous span.
///
/// This means that it is possible for end_ns - start_n
/// to not be equal to delta.
pub fn end_collapse<S: Into<StrCow>>(name: S) -> u64 {
    end_impl(name, false)
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
        for past in &library.past {
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

pub fn dump_stdout() {
    pub fn print_span(span: &Span) {
        let mut buf = String::new();
        for _ in 0 .. span.depth {
            buf.push_str("  ");
        }
        buf.push_str("| ");
        buf.push_str(&format!("{}: {} ({}ms)", span.name, span.delta, span.delta as f32 / 1000000.0));
        println!("{}", buf);
        for child in &span.children {
            print_span(child);
        }
    }

    for frame in frames() {
        for span in frame.roots {
            print_span(&span);
        }
    }
}

pub use svg::dump_svg;
pub use html::dump_html;
