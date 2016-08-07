#![allow(unused)]

#![cfg_attr(feature="json", feature(custom_derive, plugin))]
#![cfg_attr(feature="json", plugin(serde_macros))]

#[macro_use]
extern crate lazy_static;
extern crate thread_id;
#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
extern crate serde_json;

mod html;

use std::cell::{RefCell, Cell};
use std::iter::Peekable;
use std::borrow::Cow;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub type StrCow = Cow<'static, str>;

lazy_static!(static ref ALL_THREADS: Mutex<Vec<(usize, Option<String>, PrivateFrame)>> = Mutex::new(Vec::new()););
thread_local!(static LIBRARY: RefCell<Library> = RefCell::new(Library::new()));

#[derive(Debug)]
struct Library {
    current: PrivateFrame,
    epoch: Instant,
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

/// A named timespan.
///
/// The span is the most important feature of Flame.  It denotes
/// a chunk of time that is important to you.
///
/// The Span records
/// * Start and stop time
/// * A list of children (also called sub-spans)
/// * A list of notes
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
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
    #[cfg_attr(feature = "json", serde(skip_serializing))]
    _priv: (),
}

/// A note for use in debugging.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Note {
    /// A short name describing what happened at some instant in time
    pub name: StrCow,
    /// A longer description
    pub description: Option<StrCow>,
    /// The time that the note was added
    pub instant: u64,
    #[cfg_attr(feature = "json", serde(skip_serializing))]
    _priv: (),
}

/// A collection of events that happened on a single thread.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Thread {
    pub id: usize,
    pub name: Option<String>,
    pub spans: Vec<Span>,
    #[cfg_attr(feature = "json", serde(skip_serializing))]
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

fn ns_since_epoch(epoch: Instant) -> u64 {
    let elapsed = epoch.elapsed();
    elapsed.as_secs() * 1000_000_000 + elapsed.subsec_nanos() as u64
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

impl Span {
    #[cfg(feature = "json")]
    pub fn into_json(&self) -> String {
        ::serde_json::to_string_pretty(self).unwrap()
    }
}

impl Thread {
    #[cfg(feature = "json")]
    pub fn into_json(&self) -> String {
        ::serde_json::to_string_pretty(self).unwrap()
    }

    #[cfg(feature = "json")]
    pub fn into_json_list(threads: &Vec<Thread>) -> String {
        ::serde_json::to_string_pretty(threads).unwrap()
    }
}

impl Library {
    fn new() -> Library {
        Library {
            current: PrivateFrame {
                all: vec![],
                id_stack: vec![],
                next_id: 0,
            },
            epoch: Instant::now(),
        }
    }
}

fn commit_impl(library: &mut Library) {
    let mut frame = PrivateFrame {
        all: vec![],
        id_stack: vec![],
        next_id: 0,
    };
    ::std::mem::swap(&mut frame, &mut library.current);

    let mut handle = ALL_THREADS.lock().unwrap();
    let thread_name = ::std::thread::current().name().map(Into::into);
    let thread_id = ::thread_id::get();
    handle.push((thread_id, thread_name, frame))
}

pub fn commit_thread() {
    LIBRARY.with(|library| commit_impl(&mut *library.borrow_mut()));
    println!("committing");
}

impl Drop for Library {
    fn drop(&mut self) {
        commit_impl(self);
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
        let epoch = library.epoch;

        let collector = &mut library.current;
        let id = collector.next_id;
        collector.next_id += 1;

        let this = Event {
            id: id,
            parent: collector.id_stack.last().cloned(),
            name: name.into(),
            collapse: false,
            start_ns: ns_since_epoch(epoch),
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
        let epoch = library.epoch;
        let collector = &mut library.current;

        let current_id = match collector.id_stack.pop() {
            Some(id) => id,
            None => panic!("flame::end({:?}) called without a currently running span!",
                          &name)
        };
        let event = &mut collector.all[current_id as usize];

        if event.name != name {
            panic!("flame::end({}) attempted to end {}", &name, event.name);
        }

        let timestamp = ns_since_epoch(epoch);
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

/// Ends the current Span and returns a given result.
///
/// This is mainly useful for code generation / plugins where
/// wrapping all returned expressions is easier than creating
/// a temporary variable to hold the result.
pub fn end_with<S: Into<StrCow>, R>(name: S, result: R) -> R {
    end_impl(name, false);
    result
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
        let epoch = library.epoch;

        let collector = &mut library.current;

        let current_id = match collector.id_stack.last() {
            Some(id) => *id,
            None => panic!("flame::note({}, {:?}) called without a currently running span!",
                           &name, &description)
        };

        let event = &mut collector.all[current_id as usize];
        event.notes.push(Note {
            name: name,
            description: description,
            instant: ns_since_epoch(epoch),
            _priv: ()
        });
    });
}

/// Clears all of the recorded info that Flame has
/// tracked.
pub fn clear() {
    LIBRARY.with(|library| {
        let mut library = library.borrow_mut();
        library.current = PrivateFrame {
            all: vec![],
            id_stack: vec![],
            next_id: 0,
        };
        library.epoch = Instant::now();
    });

    let mut handle = ALL_THREADS.lock().unwrap();
    handle.clear();
}

/// Returns a list of spans from the current thread
pub fn spans() -> Vec<Span> {
    LIBRARY.with(|library| {
        let library = library.borrow();
        let cur = &library.current;
        convert_events_to_span(cur.all.iter())
    })
}

pub fn threads() -> Vec<Thread> {
    let mut handle = ALL_THREADS.lock().unwrap();

    let my_thread_name = ::std::thread::current().name().map(Into::into);
    let my_thread_id = ::thread_id::get();

    let mut out = vec![ Thread {
        id: my_thread_id,
        name: my_thread_name,
        spans: spans(),
        _priv: (),
    }];

    for &(id, ref name, ref frm) in &*handle {
        out.push(Thread {
            id: id,
            name: name.clone(),
            spans: convert_events_to_span(frm.all.iter()),
            _priv: (),
        });
    }

    out
}

/// Prints all of the frames to stdout.
pub fn debug() {
    LIBRARY.with(|library| {
        println!("{:?}", library);
    });
}

pub fn dump_stdout() {
    fn print_span(span: &Span) {
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

    for span in spans() {
        print_span(&span);
    }
}

pub use html::dump_html;
