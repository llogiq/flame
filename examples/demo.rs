extern crate flame;

pub fn main() {
    flame::start("rendering");
    flame::note("something happened!", None);

    flame::end("rendering");

    flame::debug();
}

