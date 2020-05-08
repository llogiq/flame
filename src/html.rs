use std::io::Write;
use std::io::Result as IoResult;
use super::{Span};

pub fn dump_html_custom<W: Write>(mut out: W, spans: &[Span]) -> IoResult<()> {
    fn dump_spans<W: Write>(out: &mut W, span: &Span) -> IoResult<()> {
        writeln!(out, "{{")?;
        writeln!(out, r#"name: {:?},"#, span.name)?;
        writeln!(out, "value: {},", span.delta)?;
        writeln!(out, "start: {},", span.start_ns)?;
        writeln!(out, "end: {},", span.end_ns)?;
        writeln!(out, "children: [")?;
        for child in &span.children {
            dump_spans(out, child)?;
            writeln!(out, ",")?;
        }
        writeln!(out, "],")?;
        writeln!(out, "}}")?;
        Ok(())
    }

    write!(out, r#"
<!doctype html>
<html>
    <head>
        <meta charset="utf-8">
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
            var flamegraph =
                d3.flameGraph()
                  .width(width)
                  .height(height)
                  .tooltip(false)
                  .sort(function(a, b){{
                    if (a.start < b.start) {{
                        return -1;
                    }} else if (a.start > b.start) {{
                        return 1;
                    }} else {{
                        return 0;
                    }}
                  }});
            d3.select("body").datum({{ children: [
"#, include_str!("../resources/flameGraph.css"), include_str!("../resources/d3.js"), include_str!("../resources/d3-tip.js"), include_str!("../resources/flameGraph.js"))?;

    for span in spans {
        dump_spans(&mut out, &span)?;
        writeln!(out, ",")?;
    }

    write!(out, r#"]}}).call(flamegraph);
         </script>
    </body>
</html>"#)?;

    Ok(())
}

pub fn dump_html<W: Write>(out: W) -> IoResult<()> {
    dump_html_custom(out, &::spans())
}
