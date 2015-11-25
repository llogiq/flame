use std::io::Write;
use std::io::Result as IoResult;

use super::{Span, Frame, frames};

static BAR_HEIGHT: usize = 20;

struct DumpContext {
    min_timestamp: u64,
    max_timestamp: u64,
    max_depth: u16,

    // used to give each span a unique identifier
    id: u32
}

impl DumpContext {
    fn new(frames: &[Frame]) -> Option<DumpContext> {
        let mut min_timestamp;
        let mut max_timestamp;
        let mut max_depth = 0;
        if let Some(first) = frames.first().and_then(|frame| frame.roots.first()) {
            min_timestamp = first.start_ns;
        } else {
            return None;
        }
        if let Some(last) = frames.last().and_then(|frame| frame.roots.last()) {
            max_timestamp = last.end_ns;
        } else {
            return None;
        }
        for frame in frames {
            for event in &frame.roots {
                DumpContext::get_depth(event, &mut max_depth);
            }
        }

        Some(DumpContext{
            min_timestamp: min_timestamp,
            max_timestamp: max_timestamp,
            max_depth: max_depth,
            id: 0,
        })
    }

    fn get_depth(span: &Span, cur_max: &mut u16) {
        if span.depth > *cur_max {
            *cur_max = span.depth;
        }

        for child in &span.children {
            DumpContext::get_depth(child, cur_max);
        }
    }

    fn compute_x_percent(&self, x: u64) -> f64 {
        ((x - self.min_timestamp) as f64) / ((self.max_timestamp - self.min_timestamp) as f64)
    }

    fn compute_y_percent(&self, depth: u16) -> f64 {
        (depth as f64) / ((self.max_depth + 1) as f64)
    }
}

pub fn dump_svg<W: Write>(out: &mut W) -> IoResult<()> {
    let frames = frames();
    if let Some(mut dump_ctx) = DumpContext::new(&frames) {
        do_dump_svg(&frames, out, &mut dump_ctx)
    } else {
        // TODO: emit an svg that has this error in it
        panic!("no frames to inspect.");
    }
}

fn do_dump_svg<W: Write>(frames: &[Frame], out: &mut W, ctx: &mut DumpContext) -> IoResult<()> {
    let h = ctx.compute_y_percent(1) * 100.0;
    try!(write!(out, r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="20%">"#));
    try!(write!(out,
r#"<defs>
    <linearGradient id="hidegrad">
        <stop stop-color="white" offset="0%"/>
        <stop stop-color="white" offset="70%"/>
        <stop stop-color="black" offset="90%"/>
        <stop stop-color="black" offset="100%"/>
    </linearGradient>
</defs>"#));

    try!(write!(out,
r#"
<style>
rect:hover + text {{
    mask: none;
}}

.label {{
    font-size: {}px;
    fill: white;
}}

.label:hover {{
    mask: none;
}}

.mask {{
    fill: url(#hidegrad);
}}
</style>"#, h / 2.0));

    for frame in frames {
        for span in &frame.roots {
            dump_span(span, out, ctx);
        }
    }
    try!(write!(out, r#"</svg>"#));
    Ok(())
}

fn dump_span<W: Write>(span: &Span, out: &mut W, ctx: &mut DumpContext) -> IoResult<()> {
    let x = ctx.compute_x_percent(span.start_ns) * 100.0;
    let y = 100.0 - ctx.compute_y_percent(span.depth + 1) * 100.0;
    let w = (ctx.compute_x_percent(span.end_ns) - ctx.compute_x_percent(span.start_ns)) * 100.0;
    let h = ctx.compute_y_percent(1) * 100.0;
    let color = depth_to_rgb(span.depth, ctx.max_depth);
    let id = ctx.id;
    ctx.id += 1;

    try!(write!(out, r#"<mask id="mask{}"><rect class="mask" x="{}%" y="{}%" width="{}%" height="{}%" /> </mask>"#,
                id, x, y, w, h));
    try!(write!(out, r#"<rect x="{}%" y="{}%" width="{}%" height="{}%" fill="rgb{:?}"/>"#,
                x, y, w, h, color));
    try!(write!(out, r#"<text class="label" x="{}%" y="{}%" width="{}%" mask="url(#mask{})"> {} </text>"#,
                x + 1.0, y + h / 1.5, w, id, span.name));

    for child in &span.children {
        dump_span(child, out, ctx);
    }
    Ok(())
}

fn depth_to_rgb(depth: u16, max_depth: u16) -> (u8, u8, u8) {
    let percent = (depth as f32) / (max_depth as f32);
    let red = (percent * (255.0)) as u8;
    (red, 0, 0)
}
