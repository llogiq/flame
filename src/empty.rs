//! Here's an example of how to use some of FLAMEs APIs:
//!
//! ```
//! extern crate flame;
//!
//! use std::fs::File;
//!
//! pub fn main() {
//!     // Manual `start` and `end`
//!     flame::start("read file");
//!     let x = read_a_file();
//!     flame::end("read file");
//!
//!     // Time the execution of a closure.  (the result of the closure is returned)
//!     let y = flame::span_of("database query", || query_database());
//!
//!     // Time the execution of a block by creating a guard.
//!     let z = {
//!         let _guard = flame::start_guard("cpu-heavy calculation");
//!         cpu_heavy_operations_1();
//!         // Notes can be used to annotate a particular instant in time.
//!         flame::note("something interesting happened", None);
//!         cpu_heavy_operations_2()
//!     };
//!
//!     // Dump the report to disk
//!     flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
//!
//!     // Or read and process the data yourself!
//!     let spans = flame::spans();
//!
//!     println!("{} {} {}", x, y, z);
//! }
//!
//! # fn read_a_file() -> bool { true }
//! # fn query_database() -> bool { true }
//! # fn cpu_heavy_operations_1() {}
//! # fn cpu_heavy_operations_2() -> bool { true }
//! ```

use std;

use std::borrow::Cow;
use std::io::{Error as IoError, Write};

pub type StrCow = Cow<'static, str>;

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
    #[cfg_attr(feature = "json", serde(skip_serializing))] collapsable: bool,
    #[cfg_attr(feature = "json", serde(skip_serializing))] _priv: (),
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
    #[cfg_attr(feature = "json", serde(skip_serializing))] _priv: (),
}

/// A collection of events that happened on a single thread.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Thread {
    pub id: usize,
    pub name: Option<String>,
    pub spans: Vec<Span>,
    #[cfg_attr(feature = "json", serde(skip_serializing))] _priv: (),
}

pub struct SpanGuard {}

impl Drop for SpanGuard {
    fn drop(&mut self) {}
}

impl SpanGuard {
    pub fn end(self) {}
    pub fn end_collapse(self) {}
}


impl Span {
    #[cfg(feature = "json")]
    pub fn into_json(&self) -> String {
        panic!();
    }
}

impl Thread {
    #[cfg(feature = "json")]
    pub fn into_json(&self) -> String {
        panic!();
    }

    #[cfg(feature = "json")]
    pub fn into_json_list(_threads: &Vec<Thread>) -> String {
        panic!();
    }
}

/// Starts a `Span` and also returns a `SpanGuard`.
///
/// When the `SpanGuard` is dropped (or `.end()` is called on it),
/// the span will automatically be ended.
pub fn start_guard<S: Into<StrCow>>(_name: S) -> SpanGuard {
    SpanGuard {}
}

/// Starts and ends a `Span` that lasts for the duration of the
/// function `f`.
pub fn span_of<S, F, R>(_name: S, f: F) -> R
where
    S: Into<StrCow>,
    F: FnOnce() -> R,
{
    f()
}

/// Starts a new Span
pub fn start<S: Into<StrCow>>(_name: S) {}


/// Ends the current Span and returns the number
/// of nanoseconds that passed.
pub fn end<S: Into<StrCow>>(_name: S) -> u64 {
    0
}

/// Ends the current Span and returns a given result.
///
/// This is mainly useful for code generation / plugins where
/// wrapping all returned expressions is easier than creating
/// a temporary variable to hold the result.
pub fn end_with<S: Into<StrCow>, R>(_name: S, result: R) -> R {
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
pub fn end_collapse<S: Into<StrCow>>(_name: S) -> u64 {
    0
}

/// Records a note on the current Span.
pub fn note<S: Into<StrCow>>(_name: S, _description: Option<S>) {}

/// Clears all of the recorded info that Flame has
/// tracked.
pub fn clear() {}

/// Returns a list of spans from the current thread
pub fn spans() -> Vec<Span> {
    vec![]
}

pub fn threads() -> Vec<Thread> {
    vec![]
}

/// Prints all of the frames to stdout.
pub fn debug() {}

pub fn dump_text_to_writer<W: Write>(_out: W) -> Result<(), IoError> {
    Ok(())
}

pub fn dump_stdout() {}

#[cfg(feature = "json")]
pub fn dump_json<W: std::io::Write>(_out: &mut W) -> std::io::Result<()> {
    Ok(())
}

pub use html::{dump_html, dump_html_custom};
