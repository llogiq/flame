extern crate flame;

#[test]
fn single_event() {
    flame::clear();
    flame::start("event1");
    flame::end("event1");

    let frames = flame::frames();
    assert!(frames.len() == 1);
    let frame = &frames[0];
    assert!(frame.roots.len() == 1);
    assert!(frame.roots[0].name == "event1");
}

#[test]
fn single_nested() {
    flame::clear();
    flame::start("event1");
        flame::start("event2");
        flame::end("event2");
    flame::end("event1");

    let frames = flame::frames();
    assert!(frames.len() == 1);
    let frame = &frames[0];
    assert!(frame.roots.len() == 1);
    assert!(frame.roots[0].name == "event1");
    let first = &frame.roots[0];
    assert!(first.children.len() == 1);
    assert!(first.children[0].name == "event2");
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

    let frames = flame::frames();
    assert!(frames.len() == 1);
    let frame = &frames[0];
    assert!(frame.roots.len() == 1);
    assert!(frame.roots[0].name == "event1");

    let first = &frame.roots[0];
    assert!(first.children.len() == 2);
    assert!(first.children[0].name == "event2");
    assert!(first.children[1].name == "event3");

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
    flame::note("hi", None);
}
