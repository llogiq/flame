extern crate flame;

use std::fs::File;

pub fn main() {
    flame::start("update");
        flame::start("process inputs");
        flame::end("process inputs");

        flame::start("physics");
            flame::start("broad phase");
            flame::end("broad phase");

            flame::start("narrow phase");
            flame::end("narrow phase");
        flame::end("physics");

        flame::start("network sync");
        flame::end("network sync");
    flame::end("update");

    flame::start("render");
        flame::start("build display lists");
        flame::end("build display lists");

        flame::start("draw calls");
        flame::end("draw calls");
    flame::end("render");

    flame::dump_html(&mut File::create("out.html").unwrap()).unwrap();
    flame::dump_json(&mut File::create("out.json").unwrap()).unwrap();
    flame::dump_stdout();
}
