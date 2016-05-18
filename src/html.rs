use std::io::Write;
use std::io::Result as IoResult;
use super::{Span, Frame, frames};

pub fn dump_html<W: Write>(out: &mut W) -> IoResult<()> {
    fn dump_spans<W: Write>(out: &mut W, span: &Span) -> IoResult<()> {
        try!(writeln!(out, "{{"));
        try!(writeln!(out, r#"name: "{}","#, span.name));
        try!(writeln!(out, "value: {},", span.delta));
        try!(writeln!(out, "children: ["));
        for child in &span.children {
            try!(dump_spans(out, child));
            try!(writeln!(out, ","));
        }
        try!(writeln!(out, "],"));
        try!(writeln!(out, "}}"));
        Ok(())
    }

    try!(write!(out, r#"
<!doctype html>
<html>
    <head>
        <style>
            html, body {{
                width: 100%;
                height: 100%;
                margin: 0;
                padding: 0;
            }}
            {}
        </style>
        <script>
            {}
            {}
            {}
        </script>
    </head>
    <body>
        <script>
            var width = document.body.offsetWidth;
            var height = document.body.offsetHeight - 100;
            var flamegraph = d3.flameGraph().width(width).height(height).tooltip(false);
            d3.select("body").datum({{ children: [
"#, include_str!("../resources/flameGraph.css"), include_str!("../resources/d3.js"), include_str!("../resources/d3-tip.js"), include_str!("../resources/flameGraph.js")));

    for frame in ::frames() {
        for span in &frame.roots {
            try!(dump_spans(out, span));
            try!(writeln!(out, ","));
        }
    }

    try!(write!(out, r#"]}}).call(flamegraph);
         </script>
    </body>
</html>"#));

    Ok(())
}
