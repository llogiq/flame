extern crate flame;

#[test]
fn single_event() {
    flame::clear();
    flame::start("event1");
    flame::end("event1");
}

#[test]
fn single_nested() {
    flame::clear();
    flame::start("event1");
        flame::start("event2");
        flame::end("event2");
    flame::end("event1");
}

#[test]
fn double_nested() {
    flame::clear();
    flame::start("event1");
        flame::start("event2");
        flame::end("event2");
        flame::start("event3");
        flame::end("event3");
    flame::end("event1");
}

#[test]
#[should_panic]
fn wrong_name() {
    flame::clear();
    flame::start("a");
    flame::end("b");
}

#[test]
#[should_panic]
fn cant_note() {
    flame::clear();
    flame::note("hi");
}
