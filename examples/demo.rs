extern crate flame;

pub fn main() {
    flame::start("rendering");

    flame::end("rendering");

    flame::debug();
}

